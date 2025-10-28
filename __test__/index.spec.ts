import test from 'ava'
import { readFileSync, readdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  improvedSlideMatch,
  slideMatch,
} from '../index'

const __filename = fileURLToPath(import.meta.url)
const TEST_DIR = dirname(__filename)
const IMAGES_DIR = join(TEST_DIR, 'images')

// 解析pos.txt文件
function parsePosFile(): Map<number, any> {
  const posFile = join(IMAGES_DIR, 'pos.txt')
  const content = readFileSync(posFile, 'utf-8')
  const results = new Map()

  content.split('\n').forEach(line => {
    line = line.trim()
    if (!line) return

    // 解析格式: 1{ target: [ 149, 95, 204, 140 ], target_x: 0, target_y: 0 }
    const match = line.match(/(\d+)\{ target: \[ (\d+), (\d+), (\d+), (\d+) \], target_x: (\d+), target_y: (\d+) \}/)
    if (match) {
      const [, index, x1, y1, x2, y2, target_x, target_y] = match
      results.set(parseInt(index), {
        x1: parseInt(x1),
        y1: parseInt(y1),
        x2: parseInt(x2),
        y2: parseInt(y2),
        target_x: parseInt(target_x),
        target_y: parseInt(target_y)
      })
    }
  })

  return results
}

// 获取测试用例
function getTestCases(): Array<{index: number, cut: string, bg: string, expected: any}> {
  const expectedResults = parsePosFile()
  const files = readdirSync(IMAGES_DIR).filter(f => f.endsWith('.png'))

  const cutFiles = files.filter(f => f.startsWith('cut')).sort()
  const bgFiles = files.filter(f => f.startsWith('bg')).sort()

  return cutFiles.map((cutFile) => {
    const index = parseInt(cutFile.match(/\d+/)?.[0] || '0')
    const bgFile = bgFiles.find(f => f.includes(index.toString()))

    return {
      index,
      cut: join(IMAGES_DIR, cutFile),
      bg: bgFile ? join(IMAGES_DIR, bgFile) : '',
      expected: expectedResults.get(index)
    }
  }).filter(tc => tc.bg && tc.expected)
}

const testCases = getTestCases()

// 原版算法测试
testCases.forEach(({ index, cut, bg, expected }) => {
  test(`原版算法 - 测试用例 ${index}`, (t) => {
    const targetBuffer = readFileSync(cut)
    const backgroundBuffer = readFileSync(bg)
    const bbox = slideMatch(targetBuffer, backgroundBuffer)

    const error_x1 = Math.abs(bbox.x1 - expected.x1)
    const error_y1 = Math.abs(bbox.y1 - expected.y1)
    const error_x2 = Math.abs(bbox.x2 - expected.x2)
    const error_y2 = Math.abs(bbox.y2 - expected.y2)

    console.log(`\n测试 ${index} - 原版算法:`)
    console.log(`预期: [${expected.x1}, ${expected.y1}, ${expected.x2}, ${expected.y2}]`)
    console.log(`实际: [${bbox.x1}, ${bbox.y1}, ${bbox.x2}, ${bbox.y2}]`)
    console.log(`误差: [${error_x1}, ${error_y1}, ${error_x2}, ${error_y2}]`)

    t.truthy(bbox)
    t.true(error_x1 <= 5, `x1误差应该<=5, 实际=${error_x1}`)
    t.true(error_y1 <= 5, `y1误差应该<=5, 实际=${error_y1}`)
    t.true(error_x2 <= 5, `x2误差应该<=5, 实际=${error_x2}`)
    t.true(error_y2 <= 5, `y2误差应该<=5, 实际=${error_y2}`)
  })
})

// 改进版算法测试
testCases.forEach(({ index, cut, bg, expected }) => {
  test(`改进版算法 - 测试用例 ${index}`, (t) => {
    const targetBuffer = readFileSync(cut)
    const backgroundBuffer = readFileSync(bg)
    const bbox = improvedSlideMatch(targetBuffer, backgroundBuffer)

    const error_x1 = Math.abs(bbox.x1 - expected.x1)
    const error_y1 = Math.abs(bbox.y1 - expected.y1)
    const error_x2 = Math.abs(bbox.x2 - expected.x2)
    const error_y2 = Math.abs(bbox.y2 - expected.y2)

    console.log(`\n测试 ${index} - 改进版算法:`)
    console.log(`预期: [${expected.x1}, ${expected.y1}, ${expected.x2}, ${expected.y2}]`)
    console.log(`实际: [${bbox.x1}, ${bbox.y1}, ${bbox.x2}, ${bbox.y2}]`)
    console.log(`误差: [${error_x1}, ${error_y1}, ${error_x2}, ${error_y2}]`)

    t.truthy(bbox)
    t.true(error_x1 <= 5, `x1误差应该<=5, 实际=${error_x1}`)
    t.true(error_y1 <= 5, `y1误差应该<=5, 实际=${error_y1}`)
    t.true(error_x2 <= 5, `x2误差应该<=5, 实际=${error_x2}`)
    t.true(error_y2 <= 5, `y2误差应该<=5, 实际=${error_y2}`)
  })
})

// 算法对比统计
test('算法准确性统计', (t) => {
  let originalAccurate = 0
  let improvedAccurate = 0
  let originalTotalError = 0
  let improvedTotalError = 0

  testCases.forEach(({ cut, bg, expected }) => {
    // 原版算法
    try {
      const targetBuffer = readFileSync(cut)
      const backgroundBuffer = readFileSync(bg)
      const originalBbox = slideMatch(targetBuffer, backgroundBuffer)
      const error = Math.abs(originalBbox.x1 - expected.x1) + Math.abs(originalBbox.y1 - expected.y1)
      originalTotalError += error
      if (error <= 5) originalAccurate++
    } catch (e) {
      // 失败不计入
    }

    // 改进版算法
    try {
      const targetBuffer = readFileSync(cut)
      const backgroundBuffer = readFileSync(bg)
      const improvedBbox = improvedSlideMatch(targetBuffer, backgroundBuffer)
      const error = Math.abs(improvedBbox.x1 - expected.x1) + Math.abs(improvedBbox.y1 - expected.y1)
      improvedTotalError += error
      if (error <= 5) improvedAccurate++
    } catch (e) {
      // 失败不计入
    }
  })

  console.log('\n=== 算法准确性统计 ===')
  console.log(`原版准确: ${originalAccurate}/${testCases.length} (${(originalAccurate/testCases.length*100).toFixed(1)}%)`)
  console.log(`改进准确: ${improvedAccurate}/${testCases.length} (${(improvedAccurate/testCases.length*100).toFixed(1)}%)`)
  console.log(`原版平均误差: ${(originalTotalError/testCases.length).toFixed(2)}`)
  console.log(`改进平均误差: ${(improvedTotalError/testCases.length).toFixed(2)}`)

  t.pass()
})
