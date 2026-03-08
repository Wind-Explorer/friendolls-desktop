<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { setSprite } from "./sprites";
  import { calculateDirection, moveTowards, clampPosition } from "./physics";
  import { updateIdle } from "./idle";

  interface Props {
    targetX: number;
    targetY: number;
    spriteUrl: string;
  }

  let { targetX, targetY, spriteUrl }: Props = $props();

  let nekoEl: HTMLDivElement;
  let animationFrameId: number;

  let nekoPos = $state({ x: 32, y: 32 });
  let frameCount = 0;
  let idleTime = 0;
  let idleAnimation: string | null = $state(null);
  let idleAnimationFrame = 0;

  let lastFrameTimestamp: number | null = null;

  function frame(timestamp: number) {
    if (!lastFrameTimestamp) {
      lastFrameTimestamp = timestamp;
    }
    if (timestamp - lastFrameTimestamp > 100) {
      lastFrameTimestamp = timestamp;
      doFrame();
    }
    animationFrameId = requestAnimationFrame(frame);
  }

  function doFrame() {
    frameCount += 1;
    const targetPos = { x: targetX, y: targetY };

    const { direction, distance } = calculateDirection(
      nekoPos.x,
      nekoPos.y,
      targetPos.x,
      targetPos.y,
    );

    if (distance < 10 || distance < 48) {
      const idleResult = updateIdle(
        nekoEl,
        nekoPos,
        targetPos,
        idleAnimation,
        idleAnimationFrame,
        idleTime,
      );
      idleAnimation = idleResult.idleAnimation;
      idleAnimationFrame = idleResult.idleAnimationFrame;
      idleTime = idleResult.idleTime;
      return;
    }

    idleAnimation = null;
    idleAnimationFrame = 0;

    if (idleTime > 1) {
      setSprite(nekoEl, "alert", 0);
      idleTime = Math.min(idleTime, 7);
      idleTime -= 1;
      return;
    }

    setSprite(nekoEl, direction, frameCount);

    const newPos = moveTowards(nekoPos.x, nekoPos.y, targetPos.x, targetPos.y);
    nekoPos = newPos;

    nekoEl.style.transform = `translate(${nekoPos.x - 16}px, ${nekoPos.y - 16}px)`;
  }

  onMount(() => {
    nekoEl.style.backgroundImage = `url(${spriteUrl})`;
    animationFrameId = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  });

  $effect(() => {
    if (nekoEl && spriteUrl) {
      nekoEl.style.backgroundImage = `url(${spriteUrl})`;
    }
  });
</script>

<div
  bind:this={nekoEl}
  class="pointer-events-none fixed z-999 size-8 select-none"
  style="width: 32px; height: 32px; position: fixed; image-rendering: pixelated;"
></div>
