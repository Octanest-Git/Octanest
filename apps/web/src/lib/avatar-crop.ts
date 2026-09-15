/** Pure crop geometry for square avatar export (unit-tested). */

export type CropFrame = {
  /** Natural image width in px. */
  imageWidth: number;
  /** Natural image height in px. */
  imageHeight: number;
  /** Viewport / crop square size in CSS px. */
  viewport: number;
  /** Zoom multiplier ≥ 1. */
  zoom: number;
  /** Pan offset of image top-left relative to viewport (CSS px). */
  offsetX: number;
  /** Pan offset of image top-left relative to viewport (CSS px). */
  offsetY: number;
};

export type SourceRect = {
  sx: number;
  sy: number;
  sw: number;
  sh: number;
};

/** Scale that makes the shorter image edge fill the viewport at zoom=1. */
export function baseCoverScale(imageWidth: number, imageHeight: number, viewport: number): number {
  if (imageWidth <= 0 || imageHeight <= 0 || viewport <= 0) return 1;
  return viewport / Math.min(imageWidth, imageHeight);
}

export function displayedSize(frame: CropFrame): { width: number; height: number } {
  const scale = baseCoverScale(frame.imageWidth, frame.imageHeight, frame.viewport) * frame.zoom;
  return {
    width: frame.imageWidth * scale,
    height: frame.imageHeight * scale,
  };
}

/** Clamp pan so the crop square stays inside the image. */
export function clampOffset(
  frame: Omit<CropFrame, "offsetX" | "offsetY"> & {
    offsetX: number;
    offsetY: number;
  },
): { offsetX: number; offsetY: number } {
  const { width, height } = displayedSize(frame);
  const minX = frame.viewport - width;
  const minY = frame.viewport - height;
  return {
    offsetX: Math.min(0, Math.max(minX, frame.offsetX)),
    offsetY: Math.min(0, Math.max(minY, frame.offsetY)),
  };
}

/** Map the viewport square back into natural image coordinates. */
export function sourceRectFromFrame(frame: CropFrame): SourceRect {
  const scale = baseCoverScale(frame.imageWidth, frame.imageHeight, frame.viewport) * frame.zoom;
  const sx = -frame.offsetX / scale;
  const sy = -frame.offsetY / scale;
  const sw = frame.viewport / scale;
  const sh = frame.viewport / scale;
  return {
    sx: Math.max(0, Math.min(frame.imageWidth - sw, sx)),
    sy: Math.max(0, Math.min(frame.imageHeight - sh, sy)),
    sw: Math.min(sw, frame.imageWidth),
    sh: Math.min(sh, frame.imageHeight),
  };
}

export async function cropImageToBlob(
  image: CanvasImageSource & { width: number; height: number },
  frame: CropFrame,
  outputSize = 512,
): Promise<Blob> {
  const rect = sourceRectFromFrame(frame);
  const canvas = document.createElement("canvas");
  canvas.width = outputSize;
  canvas.height = outputSize;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Could not create canvas context");
  }
  ctx.drawImage(image, rect.sx, rect.sy, rect.sw, rect.sh, 0, 0, outputSize, outputSize);
  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob((b) => resolve(b), "image/webp", 0.92);
  });
  if (blob) return blob;
  // Safari / older engines may lack webp encode — fall back to jpeg.
  const jpeg = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob((b) => resolve(b), "image/jpeg", 0.92);
  });
  if (!jpeg) throw new Error("Could not encode cropped image");
  return jpeg;
}
