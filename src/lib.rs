#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use anyhow::Context;

// 定义 SlideBBox 结构体
#[derive(Debug, Clone)]
pub struct SlideBBox {
    pub target_x: u32,
    pub target_y: u32,
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

impl SlideBBox {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "target_x": self.target_x,
            "target_y": self.target_y,
            "x1": self.x1,
            "y1": self.y1,
            "x2": self.x2,
            "y2": self.y2
        })
    }
}

// 滑块匹配函数（带透明背景裁剪）
fn slide_match_internal(target_image: &[u8], background_image: &[u8]) -> anyhow::Result<SlideBBox> {
    let target_image = image::load_from_memory(target_image)
        .context("无法加载目标图片")?;
    let background_image = image::load_from_memory(background_image)
        .context("无法加载背景图片")?;

    anyhow::ensure!(
        background_image.width() >= target_image.width(),
        "背景图片的宽度必须大于等于目标图片的宽度"
    );

    anyhow::ensure!(
        background_image.height() >= target_image.height(),
        "背景图片的高度必须大于等于目标图片的高度"
    );

    let target_image = target_image.to_rgba8();

    // 裁剪图片，只保留不透明部分
    let width = target_image.width();
    let height = target_image.height();
    let mut start_x = width;
    let mut start_y = height;
    let mut end_x = 0;
    let mut end_y = 0;

    for x in 0..width {
        for y in 0..height {
            let p = target_image.get_pixel(x, y);

            if p[3] != 0 {
                if x < start_x {
                    start_x = x;
                }

                if y < start_y {
                    start_y = y;
                }

                if x > end_x {
                    end_x = x;
                }

                if y > end_y {
                    end_y = y;
                }
            }
        }
    }

    let cropped_image = if start_x > end_x || start_y > end_y {
        // 没有任何不透明的像素
        target_image
    } else {
        image::imageops::crop_imm(
            &target_image,
            start_x,
            start_y,
            end_x - start_x + 1,
            end_y - start_y + 1,
        )
        .to_image()
    };

    // 图片转换到灰度图
    let target_image = image::imageops::grayscale(&cropped_image);

    // 使用 canny 进行边缘检测
    let target_image = imageproc::edges::canny(&target_image, 100.0, 200.0);
    let background_image = imageproc::edges::canny(&background_image.to_luma8(), 100.0, 200.0);

    // 模板匹配
    let result = imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
        &background_image,
        &target_image,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    ));

    Ok(SlideBBox {
        target_x: start_x,
        target_y: start_y,
        x1: result.max_value_location.0,
        y1: result.max_value_location.1,
        x2: result.max_value_location.0 + target_image.width(),
        y2: result.max_value_location.1 + target_image.height(),
    })
}

// 简单滑块匹配函数（无透明背景裁剪）
fn simple_slide_match_internal(target_image: &[u8], background_image: &[u8]) -> anyhow::Result<SlideBBox> {
    let target_image = image::load_from_memory(target_image)
        .context("无法加载目标图片")?;
    let background_image = image::load_from_memory(background_image)
        .context("无法加载背景图片")?;

    anyhow::ensure!(
        background_image.width() >= target_image.width(),
        "背景图片的宽度必须大于等于目标图片的宽度"
    );

    anyhow::ensure!(
        background_image.height() >= target_image.height(),
        "背景图片的高度必须大于等于目标图片的高度"
    );

    // 使用 canny 进行边缘检测
    let target_image = imageproc::edges::canny(&target_image.to_luma8(), 100.0, 200.0);
    let background_image = imageproc::edges::canny(&background_image.to_luma8(), 100.0, 200.0);

    // 模板匹配
    let result = imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
        &background_image,
        &target_image,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    ));

    Ok(SlideBBox {
        target_x: 0,
        target_y: 0,
        x1: result.max_value_location.0,
        y1: result.max_value_location.1,
        x2: result.max_value_location.0 + target_image.width(),
        y2: result.max_value_location.1 + target_image.height(),
    })
}

/// 滑块匹配（带透明背景裁剪）
/// 接受 Buffer 参数（支持 base64 解码后的 u8 数组）
#[napi]
pub fn slide_match(
    target_image: Buffer,
    background_image: Buffer,
) -> Result<String> {
    let target_bytes = target_image.as_ref();
    let background_bytes = background_image.as_ref();

    let result = slide_match_internal(target_bytes, background_bytes)
        .map_err(|e| Error::from_reason(format!("滑块匹配失败: {}", e)))?;

    Ok(result.to_json().to_string())
}

/// 简单滑块匹配（无透明背景裁剪）
/// 接受 Buffer 参数（支持 base64 解码后的 u8 数组）
#[napi]
pub fn simple_slide_match(
    target_image: Buffer,
    background_image: Buffer,
) -> Result<String> {
    let target_bytes = target_image.as_ref();
    let background_bytes = background_image.as_ref();

    let result = simple_slide_match_internal(target_bytes, background_bytes)
        .map_err(|e| Error::from_reason(format!("滑块匹配失败: {}", e)))?;

    Ok(result.to_json().to_string())
}
