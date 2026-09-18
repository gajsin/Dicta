<script lang="ts">
  interface Props {
    visible?: boolean;
    isListening?: boolean;
    audioLevel?: number;
  }

  let { visible = true, isListening = true, audioLevel = 0 }: Props = $props();

  let smoothedLevel = $state(0);
  const BAR_ENVELOPE = [0.25, 0.45, 0.7, 0.95, 1.2, 1.35, 1.2, 0.95, 0.7, 0.45, 0.25];
  const BASE_HEIGHTS = [3, 4.5, 6.5, 9, 11.5, 13, 11.5, 9, 6.5, 4.5, 3];
  let barHeights = $state<number[]>([...BASE_HEIGHTS]);

  $effect(() => {
    let animId: number | null = null;

    if (isListening && visible) {
      const updateFrame = () => {
        const now = performance.now() / 1000;

        // Smooth Lerp on incoming audio level (0.18 factor for organic fluid motion)
        smoothedLevel += (audioLevel - smoothedLevel) * 0.18;

        barHeights = BAR_ENVELOPE.map((env, i) => {
          // Ambient breathing sine wave for natural idle state
          const wave = Math.sin(now * 3.8 + i * 0.7) * 1.2 + Math.cos(now * 2.4 - i * 0.5) * 0.8;
          const baseHeight = BASE_HEIGHTS[i];
          const idleHeight = baseHeight + wave * 0.35;

          if (smoothedLevel < 0.03) {
            return Math.min(13, Math.max(2.5, idleHeight));
          }

          // Active voice response with dynamic gain envelope
          const dynamicGain = Math.min(1, smoothedLevel * 3.5);
          const voiceWave = Math.sin(now * 14 + i * 1.1) * 1.8;
          const activeHeight = baseHeight + dynamicGain * 6 * env + voiceWave * dynamicGain;

          return Math.min(13, Math.max(2.5, activeHeight));
        });

        animId = requestAnimationFrame(updateFrame);
      };
      animId = requestAnimationFrame(updateFrame);
    }

    return () => {
      if (animId !== null) cancelAnimationFrame(animId);
    };
  });
</script>

<div class="waveform-container" data-tauri-drag-region>
  {#each barHeights as h}
    <span
      class="wave-bar"
      style="transform: scaleY({(h / 13).toFixed(3)})"
      data-tauri-drag-region
    ></span>
  {/each}
</div>

<style>
  .waveform-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 12px;
    width: 46px;
    flex-shrink: 0;
    margin: 0;
    box-sizing: border-box;
  }

  .wave-bar {
    width: 1.5px;
    height: 12px;
    background: var(--wave-color, var(--accent-primary, #FF9500));
    opacity: 0.95;
    border-radius: 0.5px;
    flex-shrink: 0;
    transform-origin: center;
    will-change: transform;
    box-shadow: none;
  }
</style>
