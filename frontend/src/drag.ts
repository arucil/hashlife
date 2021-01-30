

let drag = false
let dragStartX = 0, dragStartY = 0
let dragX = 0, dragY = 0

export function startDrag(x0: number, y0: number) {
  drag = true
  dragStartX = x0
  dragStartY = y0
}

export function endDrag(): [number, number] {
  drag = false
  return [dragX - dragStartX, dragY - dragStartY]
}

export function isDragging(): boolean {
  return drag
}

export function move(x: number, y: number) {
  dragX = x
  dragY = y
}
