
export function doubleToU8Array(x: number): Uint8Array {
  const arr = new ArrayBuffer(8)
  const darr = new Float64Array(arr)
  darr[0] = x
  return new Uint8Array(arr).reverse()
}