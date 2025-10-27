# `slide_match`

![https://github.com/napi-rs/package-template/actions](https://github.com/napi-rs/package-template/workflows/CI/badge.svg)

> 高性能滑块匹配 Node.js 原生模块，使用 Rust + NAPI-RS 开发

## 功能特性

- 🚀 **高性能**：纯 Rust 实现，原生性能
- 🎯 **自动裁剪**：支持透明背景自动裁剪
- 🔧 **边缘检测**：基于 Canny 算法
- 📦 **零依赖**：无需安装 OpenCV 等第三方库
- 🖼️ **多格式支持**：支持 PNG、JPEG、GIF、WebP、BMP、ICO、TIFF、AVIF、EXR、HDR、QOI 等格式

## 安装

```bash
# 需要先构建
npm install
npm run build

# 或在 npm registry 发布后
npm install slide_match
```

## 支持的图片格式

由于使用 `image::load_from_memory`，模块支持以下图片格式：

| 格式 | 扩展名 | 说明 |
|------|--------|------|
| PNG | `.png` | 支持透明通道 |
| JPEG | `.jpg`, `.jpeg` | 常用格式 |
| GIF | `.gif` | 动画 GIF（会加载第一帧） |
| WebP | `.webp` | 现代格式 |
| BMP | `.bmp` | Windows 位图 |
| ICO | `.ico` | 图标文件 |
| TIFF | `.tiff`, `.tif` | 高质量格式 |
| AVIF | `.avif` | 新兴格式 |
| HDR | `.hdr` | 高动态范围 |
| EXR | `.exr` | 电影级格式 |
| QOI | `.qoi` | 快速无损格式 |
| PNM | `.pbm`, `.pgm`, `.ppm` | Netpbm 格式 |
| DDS | `.dds` | DirectDraw Surface |

**不支持**：TGA 格式

> 提示：所有格式的图片数据以 Buffer（u8 数组）形式传入，base64 解码后的数据同样支持。

## API 使用

### 滑块匹配

```typescript
import { slideMatch, simpleSlideMatch } from 'slide_match'

// 完整滑块匹配（带透明背景裁剪）
// Buffer 参数可以是 base64 解码后的图片数据
const targetBuffer = Buffer.from(base64String, 'base64')
const backgroundBuffer = Buffer.from(base64String2, 'base64')

const result = slideMatch(targetBuffer, backgroundBuffer)
const bbox = JSON.parse(result)
// 返回: { target_x, target_y, x1, y1, x2, y2 }

// 简单滑块匹配（无透明背景裁剪）
const simpleResult = simpleSlideMatch(targetBuffer, backgroundBuffer)
```

### Node.js 使用示例

```javascript
const fs = require('fs')
const { slideMatch } = require('slide_match')

// 方式1: 从文件读取
const targetImage = fs.readFileSync('./target.png')
const backgroundImage = fs.readFileSync('./background.png')

// 执行匹配
const result = slideMatch(targetImage, backgroundImage)
const bbox = JSON.parse(result)

console.log('匹配结果:', bbox)

// 方式2: 从 base64 字符串
const base64String = 'data:image/png;base64,iVBORw0KGgoAAAANS...'
const base64Data = base64String.split(',')[1] // 移除 data URL 前缀
const imageBuffer = Buffer.from(base64Data, 'base64')

const result2 = slideMatch(imageBuffer, backgroundImage)
const bbox2 = JSON.parse(result2)
```

### 返回值格式

```json
{
  "target_x": 10,  // 目标图片裁剪起始 X（简单匹配为 0）
  "target_y": 20,  // 目标图片裁剪起始 Y（简单匹配为 0）
  "x1": 100,        // 匹配区域左上角 X
  "y1": 200,        // 匹配区域左上角 Y
  "x2": 150,        // 匹配区域右下角 X
  "y2": 250         // 匹配区域右下角 Y
}
```

## 开发

### 前置要求

- **Rust** (最新版本)
- **Node.js** (>= 12.22.0)
- **npm/yarn**

### 构建和测试

```bash
# 安装依赖
npm install

# 构建发布版本
npm run build

# 构建调试版本
npm run build:debug

# 运行测试
npm test

# 查看基准测试
npm run bench
```

```
slide_match/
├── src/
│   └── lib.rs          # Rust 源代码
├── __test__/           # 测试文件
├── benchmark/          # 性能基准测试
├── Cargo.toml          # Rust 依赖配置
└── package.json        # Node.js 配置
```

## 算法说明

### 滑块匹配流程

1. **图片加载** - 从 Buffer（u8 数组）加载图片
2. **尺寸验证** - 确保背景图 >= 目标图
3. **透明区域裁剪** (`slideMatch`) - 自动检测并裁剪透明背景
4. **灰度转换** - 转换为灰度图
5. **边缘检测** - Canny 算法（阈值: 100, 200）
6. **模板匹配** - 归一化互相关匹配
7. **返回边界框** - 包含匹配位置信息

## 技术栈

- **[NAPI-RS](https://napi.rs/)** - Node.js 原生模块开发框架
- **[Rust](https://www.rust-lang.org/)** - 底层实现语言
- **[image](https://docs.rs/image/)** - 图像处理库
- **[imageproc](https://docs.rs/imageproc/)** - 计算机视觉算法库

## 许可证

MIT
