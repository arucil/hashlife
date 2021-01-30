import { Universe, Viewport } from "hashlife-wasm"
import * as util from "./util"

export let transX = 0, transY = 0

export function setTranslate(x: number, y: number) {
  transX = x | 0
  transY = y | 0
}

export function render(
  universe: Universe,
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
) {
  ctx.fillStyle = 'white'
  ctx.fillRect(0, 0, width, height)
  ctx.translate(transX, transY)

  ctx.beginPath()
  ctx.fillStyle = 'black'
  const viewport = new Viewport(-transX, -transY, width, height)
  universe!.write_cells(viewport, (x: number, y: number, cell: number) => {
    const u8arr = util.doubleToU8Array(cell)
    const nw = u8arr[0] << 8 | u8arr[1]
    const ne = u8arr[2] << 8 | u8arr[3]
    const sw = u8arr[4] << 8 | u8arr[5]
    const se = u8arr[6] << 8 | u8arr[7]
    renderQuadrant(ctx, x, y, nw)
    renderQuadrant(ctx, x + 4, y, ne)
    renderQuadrant(ctx, x, y + 4, sw)
    renderQuadrant(ctx, x + 4, y + 4, se)
  })

  ctx.stroke()

  ctx.translate(-transX, -transY)
}

function renderQuadrant(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  quad: number,
) {
  for (let b = 1 << 15, i = 0; b !== 0; b >>= 1, ++i) {
    if (quad & b) {
      ctx.fillRect(x + (i & 3), y + (i >> 2), 1, 1)
    }
  }
}
