#![deny(clippy::all)]

use anyhow::Context;
use image::GrayImage;
use napi::bindgen_prelude::*;
use napi_derive::napi;

// 定义 SlideBBox 结构体
#[napi(object)]
#[derive(Debug, Clone)]
pub struct SlideBBox {
  pub target_x: u32,
  pub target_y: u32,
  pub x1: u32,
  pub y1: u32,
  pub x2: u32,
  pub y2: u32,
}


// 改进算法1: 自适应Canny阈值计算
fn calculate_adaptive_canny_thresholds(img: &GrayImage) -> (f32, f32) {
  let total_pixels = (img.width() * img.height()) as f32;

  // 计算图像均值
  let sum: f32 = img.iter().map(|&p| p as f32).sum();
  let mean = sum / total_pixels;

  // 计算标准差
  let variance: f32 = img
    .iter()
    .map(|&p| {
      let diff = p as f32 - mean;
      diff * diff
    })
    .sum();
  let std_dev = (variance / total_pixels).sqrt();

  // 自适应设置阈值
  // low_threshold: 均值 - 标准差
  // high_threshold: 均值 + 2*标准差
  let low_threshold = (mean - std_dev).max(0.0);
  let high_threshold = (mean + std_dev * 2.0).min(255.0);

  // 确保阈值在合理范围内
  (low_threshold.max(50.0), high_threshold.min(250.0))
}

// 改进算法2: 置信度验证
fn validate_match_result(max_value: f32, confidence_threshold: f32) -> bool {
  max_value > confidence_threshold
}

// 滑块匹配函数（带透明背景裁剪）
fn slide_match_internal(target_image: &[u8], background_image: &[u8]) -> anyhow::Result<SlideBBox> {
  let target_image = image::load_from_memory(target_image).context("无法加载目标图片")?;
  let background_image = image::load_from_memory(background_image).context("无法加载背景图片")?;

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
  let result =
    imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
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
fn simple_slide_match_internal(
  target_image: &[u8],
  background_image: &[u8],
) -> anyhow::Result<SlideBBox> {
  let target_image = image::load_from_memory(target_image).context("无法加载目标图片")?;
  let background_image = image::load_from_memory(background_image).context("无法加载背景图片")?;

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
  let result =
    imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
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
pub fn slide_match(target_image: Buffer, background_image: Buffer) -> Result<SlideBBox> {
  let target_bytes = target_image.as_ref();
  let background_bytes = background_image.as_ref();

  let result = slide_match_internal(target_bytes, background_bytes)
    .map_err(|e| Error::from_reason(format!("滑块匹配失败: {e}")))?;

  Ok(result)
}

/// 简单滑块匹配（无透明背景裁剪）
/// 接受 Buffer 参数（支持 base64 解码后的 u8 数组）
#[napi]
pub fn simple_slide_match(target_image: Buffer, background_image: Buffer) -> Result<SlideBBox> {
  let target_bytes = target_image.as_ref();
  let background_bytes = background_image.as_ref();

  let result = simple_slide_match_internal(target_bytes, background_bytes)
    .map_err(|e| Error::from_reason(format!("滑块匹配失败: {e}")))?;

  Ok(result)
}

// ========== 改进算法实现 ==========

// 改进版滑块匹配函数（带透明背景裁剪 + 自适应阈值 + 置信度验证）
// 如果改进版置信度过低，自动回退到原版算法
fn improved_slide_match_internal(
  target_image: &[u8],
  background_image: &[u8],
  confidence_threshold: f32,
) -> anyhow::Result<SlideBBox> {
  let target_image = image::load_from_memory(target_image).context("无法加载目标图片")?;
  let background_image = image::load_from_memory(background_image).context("无法加载背景图片")?;

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
  let target_gray = image::imageops::grayscale(&cropped_image);
  let background_gray = background_image.to_luma8();

  // 使用自适应阈值进行边缘检测
  let (target_low, target_high) = calculate_adaptive_canny_thresholds(&target_gray);
  let (bg_low, bg_high) = calculate_adaptive_canny_thresholds(&background_gray);

  let target_edges = imageproc::edges::canny(&target_gray, target_low, target_high);
  let background_edges = imageproc::edges::canny(&background_gray, bg_low, bg_high);

  // 模板匹配
  let result =
    imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
      &background_edges,
      &target_edges,
      imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    ));

  // 置信度验证 - 如果置信度过低，回退到原版算法
  if !validate_match_result(result.max_value, confidence_threshold) {
    // 回退到原版算法（固定阈值100, 200）
    let target_gray = image::imageops::grayscale(&cropped_image);
    let background_gray = background_image.to_luma8();
    let target_edges = imageproc::edges::canny(&target_gray, 100.0, 200.0);
    let background_edges = imageproc::edges::canny(&background_gray, 100.0, 200.0);
    let fallback_result =
      imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
        &background_edges,
        &target_edges,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
      ));

    return Ok(SlideBBox {
      target_x: start_x,
      target_y: start_y,
      x1: fallback_result.max_value_location.0,
      y1: fallback_result.max_value_location.1,
      x2: fallback_result.max_value_location.0 + target_edges.width(),
      y2: fallback_result.max_value_location.1 + target_edges.height(),
    });
  }

  Ok(SlideBBox {
    target_x: start_x,
    target_y: start_y,
    x1: result.max_value_location.0,
    y1: result.max_value_location.1,
    x2: result.max_value_location.0 + target_edges.width(),
    y2: result.max_value_location.1 + target_edges.height(),
  })
}

// 改进版简单滑块匹配函数（无透明背景裁剪 + 自适应阈值 + 置信度验证）
fn improved_simple_slide_match_internal(
  target_image: &[u8],
  background_image: &[u8],
  confidence_threshold: f32,
) -> anyhow::Result<SlideBBox> {
  let target_image = image::load_from_memory(target_image).context("无法加载目标图片")?;
  let background_image = image::load_from_memory(background_image).context("无法加载背景图片")?;

  anyhow::ensure!(
    background_image.width() >= target_image.width(),
    "背景图片的宽度必须大于等于目标图片的宽度"
  );

  anyhow::ensure!(
    background_image.height() >= target_image.height(),
    "背景图片的高度必须大于等于目标图片的高度"
  );

  let target_gray = target_image.to_luma8();
  let background_gray = background_image.to_luma8();

  // 使用自适应阈值进行边缘检测
  let (target_low, target_high) = calculate_adaptive_canny_thresholds(&target_gray);
  let (bg_low, bg_high) = calculate_adaptive_canny_thresholds(&background_gray);

  let target_edges = imageproc::edges::canny(&target_gray, target_low, target_high);
  let background_edges = imageproc::edges::canny(&background_gray, bg_low, bg_high);

  // 模板匹配
  let result =
    imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
      &background_edges,
      &target_edges,
      imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    ));

  // 置信度验证 - 如果置信度过低，回退到原版算法
  if !validate_match_result(result.max_value, confidence_threshold) {
    // 回退到原版算法（固定阈值100, 200）
    let target_gray = target_image.to_luma8();
    let background_gray = background_image.to_luma8();
    let target_edges = imageproc::edges::canny(&target_gray, 100.0, 200.0);
    let background_edges = imageproc::edges::canny(&background_gray, 100.0, 200.0);
    let fallback_result =
      imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
        &background_edges,
        &target_edges,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
      ));

    return Ok(SlideBBox {
      target_x: 0,
      target_y: 0,
      x1: fallback_result.max_value_location.0,
      y1: fallback_result.max_value_location.1,
      x2: fallback_result.max_value_location.0 + target_edges.width(),
      y2: fallback_result.max_value_location.1 + target_edges.height(),
    });
  }

  Ok(SlideBBox {
    target_x: 0,
    target_y: 0,
    x1: result.max_value_location.0,
    y1: result.max_value_location.1,
    x2: result.max_value_location.0 + target_edges.width(),
    y2: result.max_value_location.1 + target_edges.height(),
  })
}

/// 改进版滑块匹配（带透明背景裁剪 + 自适应阈值 + 置信度验证）
/// 接受 Buffer 参数（支持 base64 解码后的 u8 数组）
///
/// # 参数
/// - target_image: 目标图片 Buffer
/// - background_image: 背景图片 Buffer
/// - confidence_threshold: 置信度阈值，范围 0.0-1.0，默认 0.3
///
#[napi]
pub fn improved_slide_match(
  target_image: Buffer,
  background_image: Buffer,
  confidence_threshold: Option<f64>,
) -> Result<SlideBBox> {
  let target_bytes = target_image.as_ref();
  let background_bytes = background_image.as_ref();
  let threshold = confidence_threshold.unwrap_or(0.3) as f32;

  // 验证置信度阈值范围
  if !(0.0..=1.0).contains(&threshold) {
    return Err(Error::from_reason("置信度阈值必须在 0.0-1.0 范围内"));
  }

  let result = improved_slide_match_internal(target_bytes, background_bytes, threshold)
    .map_err(|e| Error::from_reason(format!("改进版滑块匹配失败: {e}")))?;

  Ok(result)
}

/// 改进版简单滑块匹配（无透明背景裁剪 + 自适应阈值 + 置信度验证）
/// 接受 Buffer 参数（支持 base64 解码后的 u8 数组）
///
/// # 参数
/// - target_image: 目标图片 Buffer
/// - background_image: 背景图片 Buffer
/// - confidence_threshold: 置信度阈值，范围 0.0-1.0，默认 0.3
///
#[napi]
pub fn improved_simple_slide_match(
  target_image: Buffer,
  background_image: Buffer,
  confidence_threshold: Option<f64>,
) -> Result<SlideBBox> {
  let target_bytes = target_image.as_ref();
  let background_bytes = background_image.as_ref();
  let threshold = confidence_threshold.unwrap_or(0.3) as f32;

  // 验证置信度阈值范围
  if !(0.0..=1.0).contains(&threshold) {
    return Err(Error::from_reason("置信度阈值必须在 0.0-1.0 范围内"));
  }

  let result = improved_simple_slide_match_internal(target_bytes, background_bytes, threshold)
    .map_err(|e| Error::from_reason(format!("改进版滑块匹配失败: {e}")))?;

  Ok(result)
}

/// 改进版滑块匹配 - 从文件路径
#[napi]
pub fn improved_slide_match_with_path(
  target_image_path: String,
  background_image_path: String,
  confidence_threshold: Option<f64>,
) -> Result<SlideBBox> {
  let target_bytes = std::fs::read(&target_image_path)
    .map_err(|e| Error::from_reason(format!("无法读取目标图片: {e}")))?;
  let background_bytes = std::fs::read(&background_image_path)
    .map_err(|e| Error::from_reason(format!("无法读取背景图片: {e}")))?;

  let threshold = confidence_threshold.unwrap_or(0.3) as f32;

  if !(0.0..=1.0).contains(&threshold) {
    return Err(Error::from_reason("置信度阈值必须在 0.0-1.0 范围内"));
  }

  let result = improved_slide_match_internal(&target_bytes, &background_bytes, threshold)
    .map_err(|e| Error::from_reason(format!("改进版滑块匹配失败: {e}")))?;

  Ok(result)
}

/// 改进版简单滑块匹配 - 从文件路径
#[napi]
pub fn improved_simple_slide_match_with_path(
  target_image_path: String,
  background_image_path: String,
  confidence_threshold: Option<f64>,
) -> Result<SlideBBox> {
  let target_bytes = std::fs::read(&target_image_path)
    .map_err(|e| Error::from_reason(format!("无法读取目标图片: {e}")))?;
  let background_bytes = std::fs::read(&background_image_path)
    .map_err(|e| Error::from_reason(format!("无法读取背景图片: {e}")))?;

  let threshold = confidence_threshold.unwrap_or(0.3) as f32;

  if !(0.0..=1.0).contains(&threshold) {
    return Err(Error::from_reason("置信度阈值必须在 0.0-1.0 范围内"));
  }

  let result = improved_simple_slide_match_internal(&target_bytes, &background_bytes, threshold)
    .map_err(|e| Error::from_reason(format!("改进版滑块匹配失败: {e}")))?;

  Ok(result)
}
