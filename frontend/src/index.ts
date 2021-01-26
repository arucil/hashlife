import { Universe } from "hashlife-wasm"
import breeder from "./patterns/Breeder.lif"

const universeCanvas = document.querySelector('#universe') as HTMLCanvasElement
const universePattern = document.querySelector('#universe-pattern') as HTMLSelectElement

let universe: Universe | null = null


const Patterns: { [key: string]: string } = {
  empty: `x = 1, y = 1
!
`,
  glider: `x = 3, y = 3
bo$2bo$3o!
`,
  r: `x = 3, y = 3
b2o$2o$bo!
`,
  breeder,
}

function loadPattern(patternRle: string) {
  if (universe !== null) {
    universe.free()
  }

  try {
    universe = Universe.read(patternRle)
  } catch (e) {
    console.error(e)
    alert('invalid RLE format!')
  }

  render(0, 0, universeCanvas.getContext('2d') as any)
}

universePattern.addEventListener('change', _event => {
  loadPattern(Patterns[universePattern.value])
})

function render(
  width: number,
  height: number,
  ctx: CanvasRenderingContext2D,
) {
  ctx.translate(-(width / 2 | 0), -(height / 2 | 0))

  universe!.write_cells((x: number, y: number, cell: number) => {
    const u8arr = doubleToU8Array(cell)
    const nw = u8arr[0] << 8 | u8arr[1]
    const ne = u8arr[2] << 8 | u8arr[3]
    const sw = u8arr[4] << 8 | u8arr[5]
    const se = u8arr[6] << 8 | u8arr[7]
    console.log(nw, ne, sw, se)
  })
}

function doubleToU8Array(x: number): Uint8Array {
  const arr = new ArrayBuffer(8)
  const darr = new Float64Array(arr)
  darr[0] = x
  return new Uint8Array(arr).reverse()
}