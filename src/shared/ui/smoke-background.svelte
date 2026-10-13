<script lang="ts">
interface Props {
  /** Optional extra classes for the container */
  class?: string | undefined;
}

const { class: className = '' }: Props = $props();
</script>

<div
  aria-hidden="true"
  class="pointer-events-none absolute inset-0 overflow-hidden select-none {className}"
>
  <!-- Architectural cartographic grid overlay -->
  <div
    class="absolute inset-0 opacity-[0.035] dark:opacity-[0.04]"
    style="background-image: radial-gradient(circle at 1px 1px, currentColor 1px, transparent 0); background-size: 32px 32px;"
  ></div>

  <!-- Smoke Plume 1: Flowing Astral Gold & Crimson Ember (Primary Living Drift) -->
  <div
    class="smoke-plume-ember absolute -top-[15%] -left-[10%] h-[70vw] w-[70vw] max-w-[850px] max-h-[850px] rounded-full filter blur-[90px] will-change-transform"
  ></div>

  <!-- Smoke Plume 2: Deep Midnight Obsidian & Dark Ink Swirl -->
  <div
    class="smoke-plume-ink absolute -bottom-[20%] -right-[10%] h-[80vw] w-[80vw] max-w-[950px] max-h-[950px] rounded-full filter blur-[110px] will-change-transform"
  ></div>

  <!-- Smoke Plume 3: Cosmic Indigo / Celestial Meridian Aurora (Center-right Drift) -->
  <div
    class="smoke-plume-aurora absolute top-[25%] right-[15%] h-[55vw] w-[55vw] max-w-[700px] max-h-[700px] rounded-full filter blur-[85px] will-change-transform"
  ></div>

  <!-- Smoke Plume 4: Wandering Ethereal Vapor (Slow meandering cloud) -->
  <div
    class="smoke-plume-vapor absolute bottom-[15%] left-[20%] h-[50vw] w-[50vw] max-w-[600px] max-h-[600px] rounded-full filter blur-[95px] will-change-transform"
  ></div>

  <!-- Soft vignette framing the viewport edges -->
  <div
    class="absolute inset-0 bg-radial-[circle_at_center,transparent_45%,var(--color-surface-base)_95%]"
  ></div>
</div>

<style>
  /* Fluid Ember & Gold Cloud: animates position, scale, and color gradient */
  .smoke-plume-ember {
    background: radial-gradient(
      circle,
      rgba(223, 183, 108, 0.16) 0%,
      rgba(224, 49, 68, 0.12) 40%,
      transparent 70%
    );
    animation:
      smoke-drift-flow 18s ease-in-out infinite alternate,
      smoke-color-breathe 14s ease-in-out infinite alternate;
  }

  /* Deep Midnight Obsidian Ink: slow rotating foundation */
  .smoke-plume-ink {
    background: radial-gradient(
      circle,
      rgba(15, 23, 42, 0.65) 0%,
      rgba(45, 27, 78, 0.3) 45%,
      transparent 75%
    );
    animation: smoke-drift-reverse 22s ease-in-out infinite alternate;
  }

  /* Celestial Meridian Aurora: breathing blue-indigo glow */
  .smoke-plume-aurora {
    background: radial-gradient(
      circle,
      rgba(59, 130, 246, 0.1) 0%,
      rgba(147, 51, 234, 0.08) 40%,
      transparent 70%
    );
    animation:
      smoke-pulse-float 15s ease-in-out infinite alternate,
      smoke-color-aurora 20s ease-in-out infinite alternate;
  }

  /* Wandering Ethereal Vapor */
  .smoke-plume-vapor {
    background: radial-gradient(
      circle,
      rgba(224, 49, 68, 0.08) 0%,
      rgba(223, 183, 108, 0.06) 45%,
      transparent 70%
    );
    animation: smoke-wander 25s ease-in-out infinite alternate;
  }

  /* Light Theme Adjustments */
  :global([data-theme='light']) .smoke-plume-ember {
    background: radial-gradient(
      circle,
      rgba(166, 124, 51, 0.12) 0%,
      rgba(197, 36, 54, 0.08) 40%,
      transparent 70%
    );
  }

  :global([data-theme='light']) .smoke-plume-ink {
    background: radial-gradient(
      circle,
      rgba(220, 214, 202, 0.5) 0%,
      rgba(200, 192, 178, 0.3) 50%,
      transparent 75%
    );
  }

  :global([data-theme='light']) .smoke-plume-aurora {
    background: radial-gradient(
      circle,
      rgba(59, 130, 246, 0.06) 0%,
      rgba(147, 51, 234, 0.04) 40%,
      transparent 70%
    );
  }

  /* Smooth Movement Keyframes */
  @keyframes smoke-drift-flow {
    0% {
      transform: translate3d(0, 0, 0) scale(1) rotate(0deg);
    }
    33% {
      transform: translate3d(50px, -35px, 0) scale(1.12) rotate(6deg);
    }
    66% {
      transform: translate3d(25px, 30px, 0) scale(0.95) rotate(-3deg);
    }
    100% {
      transform: translate3d(-30px, -15px, 0) scale(1.06) rotate(4deg);
    }
  }

  @keyframes smoke-drift-reverse {
    0% {
      transform: translate3d(0, 0, 0) scale(1) rotate(0deg);
    }
    50% {
      transform: translate3d(-60px, 45px, 0) scale(1.08) rotate(-7deg);
    }
    100% {
      transform: translate3d(30px, -20px, 0) scale(0.94) rotate(5deg);
    }
  }

  @keyframes smoke-pulse-float {
    0% {
      transform: translate3d(0, 0, 0) scale(0.96);
      opacity: 0.65;
    }
    50% {
      transform: translate3d(-40px, -30px, 0) scale(1.18);
      opacity: 0.95;
    }
    100% {
      transform: translate3d(20px, 25px, 0) scale(1.02);
      opacity: 0.75;
    }
  }

  @keyframes smoke-wander {
    0% {
      transform: translate3d(0, 0, 0) scale(1);
    }
    50% {
      transform: translate3d(70px, -50px, 0) scale(1.15) rotate(5deg);
    }
    100% {
      transform: translate3d(-40px, 40px, 0) scale(0.92) rotate(-5deg);
    }
  }

  /* Smooth Color Shift Transitions */
  @keyframes smoke-color-breathe {
    0% {
      filter: blur(90px) hue-rotate(0deg);
    }
    50% {
      filter: blur(105px) hue-rotate(18deg);
    }
    100% {
      filter: blur(90px) hue-rotate(-12deg);
    }
  }

  @keyframes smoke-color-aurora {
    0% {
      filter: blur(85px) hue-rotate(0deg);
    }
    50% {
      filter: blur(100px) hue-rotate(-25deg);
    }
    100% {
      filter: blur(85px) hue-rotate(15deg);
    }
  }
</style>
