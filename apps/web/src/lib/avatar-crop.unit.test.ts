import { describe, expect, it } from "@octanest/web/test-runner";
import { baseCoverScale, clampOffset, displayedSize, sourceRectFromFrame } from "./avatar-crop";

describe("avatar-crop geometry", () => {
  it("cover-scales the shorter edge to the viewport", () => {
    expect(baseCoverScale(800, 600, 280)).toBeCloseTo(280 / 600);
    expect(baseCoverScale(400, 800, 280)).toBeCloseTo(280 / 400);
  });

  it("clamps pan so the crop square stays inside the image", () => {
    const frame = {
      imageWidth: 800,
      imageHeight: 600,
      viewport: 280,
      zoom: 1,
      offsetX: 50,
      offsetY: 50,
    };
    const clamped = clampOffset(frame);
    expect(clamped.offsetX).toBeLessThanOrEqual(0);
    expect(clamped.offsetY).toBeLessThanOrEqual(0);
    const { width, height } = displayedSize({ ...frame, ...clamped });
    expect(clamped.offsetX).toBeGreaterThanOrEqual(280 - width);
    expect(clamped.offsetY).toBeGreaterThanOrEqual(280 - height);
  });

  it("maps viewport square back into image coordinates", () => {
    const frame = {
      imageWidth: 800,
      imageHeight: 600,
      viewport: 280,
      zoom: 1,
      offsetX: 0,
      offsetY: 0,
    };
    const centered = clampOffset({
      ...frame,
      offsetX: (280 - displayedSize(frame).width) / 2,
      offsetY: (280 - displayedSize(frame).height) / 2,
    });
    const rect = sourceRectFromFrame({ ...frame, ...centered });
    expect(rect.sw).toBeCloseTo(600);
    expect(rect.sh).toBeCloseTo(600);
    expect(rect.sx).toBeGreaterThanOrEqual(0);
    expect(rect.sy).toBeGreaterThanOrEqual(0);
    expect(rect.sx + rect.sw).toBeLessThanOrEqual(800 + 1e-6);
  });
});
