export interface Position {
  x: number;
  y: number;
}

export interface RectLike {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/**
 * 判断物理像素坐标在考虑 DPI 缩放后，是否处于指定的 DOM 矩形区域（逻辑像素）内部
 */
export function isPhysicalPositionOverRect(
  position: Position | null | undefined,
  rect: RectLike | null | undefined,
  devicePixelRatio = 1,
): boolean {
  if (!position || !rect) return false;
  const scale = devicePixelRatio > 0 ? devicePixelRatio : 1;
  const logicalX = position.x / scale;
  const logicalY = position.y / scale;

  return (
    logicalX >= rect.left &&
    logicalX <= rect.right &&
    logicalY >= rect.top &&
    logicalY <= rect.bottom
  );
}

/**
 * 判断物理像素坐标是否处于指定的 DOM 元素区域内
 */
export function isPhysicalPositionOverElement(
  position: Position | null | undefined,
  element: HTMLElement | null | undefined,
  devicePixelRatio = typeof window !== "undefined"
    ? window.devicePixelRatio
    : 1,
): boolean {
  if (!position || !element) return false;
  return isPhysicalPositionOverRect(
    position,
    element.getBoundingClientRect(),
    devicePixelRatio,
  );
}
