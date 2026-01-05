import { writable, get } from "svelte/store";
import { SPRITE_SETS, SPRITE_SIZE, PET_SPEED } from "../constants/pet-sprites";

export function usePetState(initialX = 32, initialY = 32) {
  const position = writable({ x: initialX, y: initialY });
  const currentSprite = writable({ x: -3, y: -3 });

  // Internal state not exposed directly
  let frameCount = 0;
  let idleTime = 0;
  let idleAnimation: string | null = null;
  let idleAnimationFrame = 0;

  function setSprite(name: string, frame: number) {
    const sprites = SPRITE_SETS[name];
    if (!sprites) return;
    const sprite = sprites[frame % sprites.length];
    currentSprite.set({
      x: sprite[0] * SPRITE_SIZE,
      y: sprite[1] * SPRITE_SIZE,
    });
  }

  function resetIdleAnimation() {
    idleAnimation = null;
    idleAnimationFrame = 0;
  }

  function handleIdle(windowWidth: number, windowHeight: number) {
    idleTime += 1;
    const currentPos = get(position);

    // every ~ 20 seconds (idleTime increments every frame, with ~10 frames/second, so ~200 frames)
    if (
      idleTime > 10 &&
      Math.floor(Math.random() * 200) == 0 &&
      idleAnimation == null
    ) {
      let availableIdleAnimations = ["sleeping", "scratchSelf"];
      if (currentPos.x < 32) {
        availableIdleAnimations.push("scratchWallW");
      }
      if (currentPos.y < 32) {
        availableIdleAnimations.push("scratchWallN");
      }
      if (currentPos.x > windowWidth - 32) {
        availableIdleAnimations.push("scratchWallE");
      }
      if (currentPos.y > windowHeight - 32) {
        availableIdleAnimations.push("scratchWallS");
      }
      idleAnimation =
        availableIdleAnimations[
          Math.floor(Math.random() * availableIdleAnimations.length)
        ];
    }

    if (!idleAnimation) {
      setSprite("idle", 0);
      return;
    }

    switch (idleAnimation) {
      case "sleeping":
        if (idleAnimationFrame < 8) {
          setSprite("tired", 0);
          break;
        }
        setSprite("sleeping", Math.floor(idleAnimationFrame / 4));
        if (idleAnimationFrame > 192) {
          resetIdleAnimation();
        }
        break;
      case "scratchWallN":
      case "scratchWallS":
      case "scratchWallE":
      case "scratchWallW":
      case "scratchSelf":
        setSprite(idleAnimation, idleAnimationFrame);
        if (idleAnimationFrame > 9) {
          resetIdleAnimation();
        }
        break;
      default:
        setSprite("idle", 0);
        return;
    }
    idleAnimationFrame += 1;
  }

  function updatePosition(
    targetX: number,
    targetY: number,
    windowWidth: number,
    windowHeight: number,
  ) {
    frameCount += 1;
    const currentPos = get(position);

    const diffX = currentPos.x - targetX;
    const diffY = currentPos.y - targetY;
    const distance = Math.sqrt(diffX ** 2 + diffY ** 2);

    // If close enough, stop moving and idle
    if (distance < PET_SPEED || distance < 48) {
      handleIdle(windowWidth, windowHeight);
      return;
    }

    // Alert behavior: pause briefly before moving if we were idling
    if (idleTime > 1) {
      setSprite("alert", 0);
      idleTime = Math.min(idleTime, 7);
      idleTime -= 1;
      return;
    }

    idleTime = 0;
    idleAnimation = null;
    idleAnimationFrame = 0;

    // Calculate direction
    let direction = "";
    direction = diffY / distance > 0.5 ? "N" : "";
    direction += diffY / distance < -0.5 ? "S" : "";
    direction += diffX / distance > 0.5 ? "W" : "";
    direction += diffX / distance < -0.5 ? "E" : "";

    // Fallback if direction is empty
    if (direction === "") direction = "idle";

    setSprite(direction, frameCount);

    // Move towards target
    position.update((p) => ({
      x: p.x - (diffX / distance) * PET_SPEED,
      y: p.y - (diffY / distance) * PET_SPEED,
    }));
  }

  return {
    position,
    currentSprite,
    updatePosition,
    // Helper to force position if needed (e.g. on mount)
    setPosition: (x: number, y: number) => position.set({ x, y }),
  };
}
