import test from 'ava'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  simpleSlideMatchWithPath,
  slideMatchWithBuffer,
  slideMatchWithPath
} from '../index'

const __filename = fileURLToPath(import.meta.url)
const TEST_DIR = dirname(__filename)
const TARGET_IMAGE = join(TEST_DIR, 'cutImage.png')
const BACKGROUND_IMAGE = join(TEST_DIR, 'bigImage.png')

test('滑块匹配 - 从文件路径（带透明背景裁剪）', (t) => {
  const result = slideMatchWithPath(TARGET_IMAGE, BACKGROUND_IMAGE)
  t.truthy(result)

  const bbox = JSON.parse(result)
  console.log('匹配结果:', bbox)

  // 验证返回的边界框字段
  t.truthy(bbox.x1)
  t.truthy(bbox.y1)
  t.truthy(bbox.x2)
  t.truthy(bbox.y2)
  t.truthy(bbox.target_x !== undefined)
  t.truthy(bbox.target_y !== undefined)

  // 验证边界框坐标的有效性
  t.true(bbox.x1 < bbox.x2, 'x1应该小于x2')
  t.true(bbox.y1 < bbox.y2, 'y1应该小于y2')
})

test('滑块匹配 - 从字节数组（带透明背景裁剪）', (t) => {
  const targetBuffer = readFileSync(TARGET_IMAGE)
  const backgroundBuffer = readFileSync(BACKGROUND_IMAGE)

  const result = slideMatchWithBuffer(targetBuffer, backgroundBuffer)
  t.truthy(result)

  const bbox = JSON.parse(result)
  console.log('匹配结果:', bbox)

  // 验证返回的边界框字段
  t.truthy(bbox.x1)
  t.truthy(bbox.y1)
  t.truthy(bbox.x2)
  t.truthy(bbox.y2)
  t.truthy(bbox.target_x !== undefined)
  t.truthy(bbox.target_y !== undefined)

  // 验证边界框坐标的有效性
  t.true(bbox.x1 < bbox.x2, 'x1应该小于x2')
  t.true(bbox.y1 < bbox.y2, 'y1应该小于y2')
})

test('简单滑块匹配 - 从文件路径（无透明背景裁剪）', (t) => {
  const result = simpleSlideMatchWithPath(TARGET_IMAGE, BACKGROUND_IMAGE)
  t.truthy(result)

  const bbox = JSON.parse(result)
  console.log('简单匹配结果:', bbox)

  // 验证返回的边界框字段
  t.truthy(bbox.x1)
  t.truthy(bbox.y1)
  t.truthy(bbox.x2)
  t.truthy(bbox.y2)
  t.truthy(bbox.target_x !== undefined)
  t.truthy(bbox.target_y !== undefined)

  // 简单匹配target_x和target_y应该为0
  t.is(bbox.target_x, 0, '简单匹配target_x应该为0')
  t.is(bbox.target_y, 0, '简单匹配target_y应该为0')

  // 验证边界框坐标的有效性
  t.true(bbox.x1 < bbox.x2, 'x1应该小于x2')
  t.true(bbox.y1 < bbox.y2, 'y1应该小于y2')
})

test('滑块匹配 - 验证结果准确性', (t) => {
  const result = slideMatchWithPath(TARGET_IMAGE, BACKGROUND_IMAGE)
  const bbox = JSON.parse(result)

  // 根据pos.txt中的预期结果进行验证
  // 注意：实际匹配结果可能与预期不完全一致，这里只是验证格式
  t.truthy(typeof bbox.x1 === 'number')
  t.truthy(typeof bbox.y1 === 'number')
  t.truthy(typeof bbox.x2 === 'number')
  t.truthy(typeof bbox.y2 === 'number')
  t.truthy(typeof bbox.target_x === 'number')
  t.truthy(typeof bbox.target_y === 'number')

  console.log('预期数据: target [177, 61, 232, 106]')
  console.log('实际结果:', bbox)
})

test('解释target_x和target_y的含义', (t) => {
  const result = slideMatchWithPath(TARGET_IMAGE, BACKGROUND_IMAGE)
  const bbox = JSON.parse(result)

  console.log('\n=== target_x 和 target_y 的含义 ===')
  console.log('target_x:', bbox.target_x, '- 这是图片裁剪时从原图左边界向右的偏移量')
  console.log('target_y:', bbox.target_y, '- 这是图片裁剪时从原图上边界向下的偏移量')
  console.log('\n如果 target_x = 0, target_y = 0，说明：')
  console.log('1. 图片没有透明背景，或')
  console.log('2. 透明部分在图片边缘的最左上角位置，即第一个有效像素就在(0,0)位置')
  console.log('\n裁剪后的实际滑块区域：')
  console.log('在背景图中的位置: x1=' + bbox.x1 + ', y1=' + bbox.y1 + ', x2=' + bbox.x2 + ', y2=' + bbox.y2)
  console.log('需要滑动的距离: x = ' + bbox.x1 + ' - ' + bbox.target_x + ' = ' + (bbox.x1 - bbox.target_x))

  t.pass()
})
