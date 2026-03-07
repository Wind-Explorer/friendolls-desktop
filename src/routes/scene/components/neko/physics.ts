import { nekoSpeed } from "./sprites";

export interface Position {
  x: number;
  y: number;
}

export function calculateDirection(
  fromX: number,
  fromY: number,
  toX: number,
  toY: number,
): { direction: string; distance: number } {
  const diffX = fromX - toX;
  const diffY = fromY - toY;
  const distance = Math.sqrt(diffX ** 2 + diffY ** 2);

  let direction = "";
  direction = diffY / distance > 0.5 ? "N" : "";
  direction += diffY / distance < -0.5 ? "S" : "";
  direction += diffX / distance > 0.5 ? "W" : "";
  direction += diffX / distance < -0.5 ? "E" : "";

  return { direction, distance };
}

export function moveTowards(
  currentX: number,
  currentY: number,
  targetX: number,
  targetY: number,
  speed: number = nekoSpeed,
): Position {
  const diffX = targetX - currentX;
  const diffY = targetY - currentY;
  const distance = Math.sqrt(diffX ** 2 + diffY ** 2);

  if (distance === 0) return { x: currentX, y: currentY };

  const newX = currentX + (diffX / distance) * speed;
  const newY = currentY + (diffY / distance) * speed;

  return clampPosition(newX, newY);
}

export function clampPosition(x: number, y: number): Position {
  const margin = 16;
  return {
    x: Math.min(Math.max(margin, x), window.innerWidth - margin),
    y: Math.min(Math.max(margin, y), window.innerHeight - margin),
  };
}
