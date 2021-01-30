import { Universe } from "hashlife-wasm"
import Patterns from "./patterns"
import { render, setTranslate, transX, transY } from "./render"
import * as drag from "./drag"

const univCanvas = document.querySelector('#universe') as HTMLCanvasElement
const univPattern = document.querySelector('#universe-pattern') as HTMLSelectElement
const zoomInput = document.querySelector('#zoom-input') as HTMLInputElement
const zoomLabel = document.querySelector('#zoom-label') as HTMLSpanElement
const generationLabel = document.querySelector('#generation-label') as HTMLSpanElement
const stepInput = document.querySelector('#step-input') as HTMLInputElement

const DEFAULT_ZOOM = 4

let universe: Universe | null = null
let curPattern = ''
let generation = 0
let zoom = 1
let step = 1

function renderUniverse() {
  render(
    universe!,
    univCanvas.getContext('2d') as CanvasRenderingContext2D,
    univCanvas.width,
    univCanvas.height)
}

function loadPattern(patternRle: string) {

  try {
    const newUniverse = Universe.read(patternRle)
    if (universe !== null) {
      universe.free()
    }
    universe = newUniverse
  } catch (e) {
    console.error(e)
    alert('invalid RLE format!')
    return
  }

  curPattern = patternRle
  setGeneration(0)
}

function setZoom(newZoom: number) {
  zoom = newZoom
  univCanvas.width = univCanvas.scrollWidth / zoom
  univCanvas.height = univCanvas.scrollHeight / zoom
  zoomLabel.textContent = (newZoom * 100 + 0.5 | 0) + '%'
}

function setGeneration(newGen: number) {
  generation = newGen
  generationLabel.textContent = String(newGen)
}

function resetTranslate() {
  setTranslate(univCanvas.width >> 1, univCanvas.height >> 1)
}

function reset() {
  loadPattern(curPattern)
  renderUniverse()
}

function evolve() {
  universe!.simulate(step)
  setGeneration(generation + step)
  renderUniverse()
}

univPattern.addEventListener('change', () => {
  loadPattern(Patterns[univPattern.value])
  renderUniverse()
})

zoomInput.addEventListener('change', () => {
  setZoom(Math.pow(2, +zoomInput.value / 10))
  renderUniverse()
})

stepInput.addEventListener('change', () => {
  step = +stepInput.value
});

univCanvas.addEventListener('mousedown', event => {
  if (event.button === 0) {
    drag.startDrag(event.pageX - univCanvas.offsetLeft, event.pageY - univCanvas.offsetTop)
  }
});

univCanvas.addEventListener('mousemove', event => {
  if (drag.isDragging()) {
    drag.move(event.pageX - univCanvas.offsetLeft, event.pageY - univCanvas.offsetTop)
  }
});

univCanvas.addEventListener('mouseup', event => {
  if (event.button === 0) {
    const [dx, dy] = drag.endDrag()
    setTranslate(transX + dx / zoom, transY + dy / zoom)
    renderUniverse()
  }
});

(document.querySelector('#reset') as HTMLButtonElement).addEventListener('click', reset);

(document.querySelector('#evolve') as HTMLButtonElement).addEventListener('click', evolve);

(document.querySelector('#reset-coord') as HTMLButtonElement).addEventListener('click', () => {
  resetTranslate()
  renderUniverse()
});

for (const pattern in Patterns) {
  const child = document.createElement('option')
  child.value = pattern
  child.textContent = pattern
  if (curPattern === '') {
    curPattern = Patterns[pattern]
  }
  univPattern.appendChild(child)
}

univCanvas.width = 600
univCanvas.height = 480
zoomInput.value = String(Math.log2(DEFAULT_ZOOM) * 10)
stepInput.value = "1"
setZoom(DEFAULT_ZOOM)
resetTranslate()
reset()