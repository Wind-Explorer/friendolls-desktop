export interface ButtonPosition {
  x: number;
  y: number;
}

export function getButtonPosition(
  index: number,
  total: number,
): ButtonPosition {
  if (total <= 1) {
    return { x: 0, y: -48 };
  }

  // A specific set of numbers for the buttons to look correct
  const startAngle = -160;
  const endAngle = -20;
  const angle = startAngle + ((endAngle - startAngle) / (total - 1)) * index;
  const angleRad = (angle * Math.PI) / 180;
  const radius = 48;

  return {
    x: Math.cos(angleRad) * radius,
    y: Math.sin(angleRad) * radius,
  };
}
