<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import onekoGif from "../../assets/oneko/oneko.gif";

  export let targetX = 0;
  export let targetY = 0;
  export let name = "";

  let nekoPosX = 32;
  let nekoPosY = 32;
  let frameCount = 0;
  let idleTime = 0;
  let idleAnimation: string | null = null;
  let idleAnimationFrame = 0;
  let currentSprite = { x: -3, y: -3 }; // idle sprite initially

  const nekoSpeed = 10;
  let animationFrameId: number;
  let lastFrameTimestamp: number;

  // Sprite constants from oneko.js
  const spriteSets: Record<string, [number, number][]> = {
    idle: [[-3, -3]],
    alert: [[-7, -3]],
    scratchSelf: [
      [-5, 0],
      [-6, 0],
      [-7, 0],
    ],
    scratchWallN: [
      [0, 0],
      [0, -1],
    ],
    scratchWallS: [
      [-7, -1],
      [-6, -2],
    ],
    scratchWallE: [
      [-2, -2],
      [-2, -3],
    ],
    scratchWallW: [
      [-4, 0],
      [-4, -1],
    ],
    tired: [[-3, -2]],
    sleeping: [
      [-2, 0],
      [-2, -1],
    ],
    N: [
      [-1, -2],
      [-1, -3],
    ],
    NE: [
      [0, -2],
      [0, -3],
    ],
    E: [
      [-3, 0],
      [-3, -1],
    ],
    SE: [
      [-5, -1],
      [-5, -2],
    ],
    S: [
      [-6, -3],
      [-7, -2],
    ],
    SW: [
      [-5, -3],
      [-6, -1],
    ],
    W: [
      [-4, -2],
      [-4, -3],
    ],
    NW: [
      [-1, 0],
      [-1, -1],
    ],
  };

  function setSprite(name: string, frame: number) {
    const sprites = spriteSets[name];
    const sprite = sprites[frame % sprites.length];
    currentSprite = { x: sprite[0] * 32, y: sprite[1] * 32 };
  }

  function resetIdleAnimation() {
    idleAnimation = null;
    idleAnimationFrame = 0;
  }

  function idle() {
    idleTime += 1;

    // every ~ 20 seconds (idleTime increments every frame, with ~10 frames/second, so ~200 frames)
    if (
      idleTime > 10 &&
      Math.floor(Math.random() * 200) == 0 &&
      idleAnimation == null
    ) {
      let availableIdleAnimations = ["sleeping", "scratchSelf"];
      if (nekoPosX < 32) {
        availableIdleAnimations.push("scratchWallW");
      }
      if (nekoPosY < 32) {
        availableIdleAnimations.push("scratchWallN");
      }
      if (nekoPosX > window.innerWidth - 32) {
        availableIdleAnimations.push("scratchWallE");
      }
      if (nekoPosY > window.innerHeight - 32) {
        availableIdleAnimations.push("scratchWallS");
      }
      idleAnimation =
        availableIdleAnimations[
          Math.floor(Math.random() * availableIdleAnimations.length)
        ];
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

  function frame(timestamp: number) {
    if (!lastFrameTimestamp) {
      lastFrameTimestamp = timestamp;
    }

    // 100ms per frame for the animation loop
    if (timestamp - lastFrameTimestamp > 100) {
      lastFrameTimestamp = timestamp;
      updatePosition();
    }

    animationFrameId = requestAnimationFrame(frame);
  }

  function updatePosition() {
    frameCount += 1;
    const diffX = nekoPosX - targetX;
    const diffY = nekoPosY - targetY;
    const distance = Math.sqrt(diffX ** 2 + diffY ** 2);

    // If close enough, stop moving and idle
    if (distance < nekoSpeed || distance < 48) {
      idle();
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

    // Fallback if direction is empty (shouldn't happen with logic above but good safety)
    if (direction === "") direction = "idle";

    setSprite(direction, frameCount);

    // Move towards target
    nekoPosX -= (diffX / distance) * nekoSpeed;
    nekoPosY -= (diffY / distance) * nekoSpeed;
  }

  onMount(() => {
    // Initialize position to target so it doesn't fly in from 32,32 every time
    nekoPosX = targetX;
    nekoPosY = targetY;

    animationFrameId = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  });
</script>

<div
  class="desktop-pet flex flex-col items-center"
  style="
    transform: translate({nekoPosX - 16}px, {nekoPosY - 16}px);
    z-index: 50;
  "
>
  <div
    class="pixelated"
    style="
      width: 32px;
      height: 32px;
      background-image: url({onekoGif});
      background-position: {currentSprite.x}px {currentSprite.y}px;
    "
  ></div>
  <span
    class="text-[10px] bg-black/50 text-white px-1 rounded backdrop-blur-sm mt-1 whitespace-nowrap"
  >
    {name}
  </span>
</div>

<style>
  .desktop-pet {
    position: fixed; /* Fixed relative to the viewport/container */
    top: 0;
    left: 0;
    pointer-events: none; /* Let clicks pass through */
    will-change: transform;
  }

  .pixelated {
    image-rendering: pixelated;
  }
</style>
