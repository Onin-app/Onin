import { describe, it, expect } from "vitest";
import {
  isPhysicalPositionOverRect,
  isPhysicalPositionOverElement,
} from "../dragDrop";

describe("dragDrop utils", () => {
  describe("isPhysicalPositionOverRect", () => {
    const rect = { left: 100, right: 300, top: 50, bottom: 250 };

    it("should return false if position or rect is missing", () => {
      expect(isPhysicalPositionOverRect(null, rect)).toBe(false);
      expect(isPhysicalPositionOverRect({ x: 150, y: 100 }, null)).toBe(false);
      expect(isPhysicalPositionOverRect(undefined, undefined)).toBe(false);
    });

    it("should handle 100% scale (devicePixelRatio = 1)", () => {
      expect(isPhysicalPositionOverRect({ x: 150, y: 100 }, rect, 1)).toBe(
        true,
      );
      expect(isPhysicalPositionOverRect({ x: 50, y: 100 }, rect, 1)).toBe(
        false,
      );
      expect(isPhysicalPositionOverRect({ x: 350, y: 100 }, rect, 1)).toBe(
        false,
      );
      expect(isPhysicalPositionOverRect({ x: 150, y: 30 }, rect, 1)).toBe(
        false,
      );
      expect(isPhysicalPositionOverRect({ x: 150, y: 260 }, rect, 1)).toBe(
        false,
      );
    });

    it("should correctly convert physical coordinates under 125% scale", () => {
      // 125% scale: physical position 250x125 => logical 200x100 (inside rect [100, 300, 50, 250])
      expect(isPhysicalPositionOverRect({ x: 250, y: 125 }, rect, 1.25)).toBe(
        true,
      );

      // physical 375x125 => logical 300x100 (boundary => true)
      expect(isPhysicalPositionOverRect({ x: 375, y: 125 }, rect, 1.25)).toBe(
        true,
      );

      // physical 400x125 => logical 320x100 (outside => false)
      expect(isPhysicalPositionOverRect({ x: 400, y: 125 }, rect, 1.25)).toBe(
        false,
      );
    });

    it("should correctly convert physical coordinates under 150% scale", () => {
      // 150% scale: physical 300x150 => logical 200x100
      expect(isPhysicalPositionOverRect({ x: 300, y: 150 }, rect, 1.5)).toBe(
        true,
      );

      // Without scale conversion, x=350 would be considered outside (350 > 300), but at 1.5 scale logical x is 233.3 => inside!
      expect(isPhysicalPositionOverRect({ x: 350, y: 150 }, rect, 1.5)).toBe(
        true,
      );
    });

    it("should fallback gracefully if devicePixelRatio is 0 or negative", () => {
      expect(isPhysicalPositionOverRect({ x: 150, y: 100 }, rect, 0)).toBe(
        true,
      );
      expect(isPhysicalPositionOverRect({ x: 150, y: 100 }, rect, -1)).toBe(
        true,
      );
    });
  });

  describe("isPhysicalPositionOverElement", () => {
    it("should return false if element is null or undefined", () => {
      expect(isPhysicalPositionOverElement({ x: 100, y: 100 }, null)).toBe(
        false,
      );
    });

    it("should call getBoundingClientRect on element", () => {
      const mockElement = {
        getBoundingClientRect: () => ({
          left: 50,
          right: 150,
          top: 50,
          bottom: 150,
          width: 100,
          height: 100,
          x: 50,
          y: 50,
          toJSON: () => {},
        }),
      } as unknown as HTMLElement;

      expect(
        isPhysicalPositionOverElement({ x: 100, y: 100 }, mockElement, 1),
      ).toBe(true);
      expect(
        isPhysicalPositionOverElement({ x: 200, y: 100 }, mockElement, 1),
      ).toBe(false);
    });
  });
});
