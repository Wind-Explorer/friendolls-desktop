import { setSprite, nekoSpeed } from "./sprites";
import type { Position } from "./physics";

export function updateIdle(
  nekoEl: HTMLDivElement,
  nekoPos: Position,
  targetPos: Position,
  idleAnimation: string | null,
  idleAnimationFrame: number,
  idleTime: number,
): {
  isIdle: boolean;
  idleAnimation: string | null;
  idleAnimationFrame: number;
  idleTime: number;
} {
  const distance = Math.sqrt(
    (nekoPos.x - targetPos.x) ** 2 + (nekoPos.y - targetPos.y) ** 2,
  );
  if (distance >= nekoSpeed && distance >= 48) {
    return { isIdle: false, idleAnimation, idleAnimationFrame, idleTime };
  }

  let newIdleTime = idleTime + 1;
  let newIdleAnimation = idleAnimation;
  let newIdleFrame = idleAnimationFrame;

  if (
    newIdleTime > 10 &&
    Math.floor(Math.random() * 200) == 0 &&
    newIdleAnimation == null
  ) {
    let availableIdleAnimations = ["sleeping", "scratchSelf"];
    if (nekoPos.x < 32) {
      availableIdleAnimations.push("scratchWallW");
    }
    if (nekoPos.y < 32) {
      availableIdleAnimations.push("scratchWallN");
    }
    if (nekoPos.x > window.innerWidth - 32) {
      availableIdleAnimations.push("scratchWallE");
    }
    if (nekoPos.y > window.innerHeight - 32) {
      availableIdleAnimations.push("scratchWallS");
    }
    newIdleAnimation =
      availableIdleAnimations[
        Math.floor(Math.random() * availableIdleAnimations.length)
      ];
  }

  switch (newIdleAnimation) {
    case "sleeping":
      if (newIdleFrame < 8) {
        setSprite(nekoEl, "tired", 0);
      } else {
        setSprite(nekoEl, "sleeping", Math.floor(newIdleFrame / 4));
      }
      if (newIdleFrame > 192) {
        newIdleAnimation = null;
        newIdleFrame = 0;
      }
      break;
    case "scratchWallN":
    case "scratchWallS":
    case "scratchWallE":
    case "scratchWallW":
    case "scratchSelf":
      setSprite(nekoEl, newIdleAnimation, newIdleFrame);
      if (newIdleFrame > 9) {
        newIdleAnimation = null;
        newIdleFrame = 0;
      }
      break;
    default:
      setSprite(nekoEl, "idle", 0);
      return { isIdle: true, idleAnimation: null, idleAnimationFrame: 0, idleTime: newIdleTime };
  }

  return {
    isIdle: true,
    idleAnimation: newIdleAnimation,
    idleAnimationFrame: newIdleFrame + 1,
    idleTime: newIdleTime,
  };
}
