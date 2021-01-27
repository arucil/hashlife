import { Universe } from "hashlife-wasm"
import Patterns from "./patterns"
import render from "./render"

const univCanvas = document.querySelector('#universe') as HTMLCanvasElement
const univPattern = document.querySelector('#universe-pattern') as HTMLSelectElement
const zoomInput = document.querySelector('#zoom-input') as HTMLInputElement
const zoomLabel = document.querySelector('#zoom-label') as HTMLSpanElement
const generationLabel = document.querySelector('#generation-label') as HTMLSpanElement
const stepInput = document.querySelector('#step-input') as HTMLInputElement

const DEFAULT_ZOOM = 4

univCanvas.width = 600
univCanvas.height = 480

let universe: Universe | null = null
let pattern = Patterns.empty
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

  pattern = patternRle
  renderUniverse()
}

function setZoom(newZoom: number) {
  zoom = newZoom
  univCanvas.width = univCanvas.scrollWidth / zoom
  univCanvas.height = univCanvas.scrollHeight / zoom
  zoomLabel.textContent = (newZoom * 100 + 0.5 | 0) + '%'
  renderUniverse()
}

function setGeneration(newGen: number) {
  generation = newGen
  generationLabel.textContent = String(newGen)
}

function reset() {
  setGeneration(0)
  loadPattern(pattern)
}

function evolve() {
  universe!.simulate(step)
  setGeneration(generation + step)
  renderUniverse()
}

univPattern.addEventListener('change', () => {
  loadPattern(Patterns[univPattern.value])
})

zoomInput.addEventListener('change', () => {
  setZoom(Math.pow(2, +zoomInput.value / 10))
})

stepInput.addEventListener('change', () => {
  step = +stepInput.value
});

(document.querySelector('#reset') as HTMLButtonElement).addEventListener('click', reset);

(document.querySelector('#evolve') as HTMLButtonElement).addEventListener('click', evolve);

reset()
zoomInput.value = String(Math.log2(DEFAULT_ZOOM) * 10)
stepInput.value = "1"
setZoom(DEFAULT_ZOOM)