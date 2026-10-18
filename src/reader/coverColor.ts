export function representativeCoverColor(pixels: Uint8ClampedArray): string | null {
  const colors = new Map<number, { weight: number; red: number; green: number; blue: number }>();
  for (let index = 0; index < pixels.length; index += 4) {
    const red = pixels[index];
    const green = pixels[index + 1];
    const blue = pixels[index + 2];
    const maximum = Math.max(red, green, blue) / 255;
    const minimum = Math.min(red, green, blue) / 255;
    const chroma = maximum - minimum;
    const lightness = (maximum + minimum) / 2;
    // 色度越高权重越大；中等明度优先，黑白灰仍保留正权重。
    const weight = (0.01 + chroma * chroma)
      * (0.1 + 0.9 * 4 * lightness * (1 - lightness)) * pixels[index + 3] / 255;
    const key = (red >> 4) << 8 | (green >> 4) << 4 | (blue >> 4);
    const bucket = colors.get(key) || { weight: 0, red: 0, green: 0, blue: 0 };
    bucket.weight += weight;
    bucket.red += red * weight;
    bucket.green += green * weight;
    bucket.blue += blue * weight;
    colors.set(key, bucket);
  }
  const dominant = [...colors.values()].sort((left, right) => right.weight - left.weight)[0];
  if (!dominant?.weight) return null;
  const channel = (value: number) => Math.round(value / dominant.weight).toString(16).padStart(2, "0");
  return `#${channel(dominant.red)}${channel(dominant.green)}${channel(dominant.blue)}`;
}
