<script lang="ts">
interface Props {
  size?: 'sm' | 'md' | 'lg' | 'xl' | undefined;
  animated?: boolean | undefined;
}

const { size = 'md', animated = false }: Props = $props();

const dimensionClass = $derived(
  {
    sm: 'w-7 h-7',
    md: 'w-10 h-10',
    lg: 'w-14 h-14',
    xl: 'w-20 h-20',
  }[size],
);
</script>

<div
  class="relative inline-flex items-center justify-center shrink-0 select-none {dimensionClass} {animated ? 'hover:scale-105 transition-transform duration-500' : ''}"
  title="Ontara"
>
  <svg
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="w-full h-full drop-shadow-[0_2px_12px_rgba(223,183,108,0.3)]"
  >
    <defs>
      <!-- Radiant Celestial Gold Gradient -->
      <linearGradient id="ontara-gold-grad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#fff0c2" />
        <stop offset="35%" stop-color="#dfb76c" />
        <stop offset="75%" stop-color="#b88334" />
        <stop offset="100%" stop-color="#7a521a" />
      </linearGradient>

      <!-- Shimmer Star Gold Gradient -->
      <linearGradient id="ontara-star-grad" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="40%" stop-color="#fcedb6" />
        <stop offset="85%" stop-color="#dfb76c" />
        <stop offset="100%" stop-color="#9a6e2a" />
      </linearGradient>

      <!-- Deep Astral Indigo / Imperial Purple Globe Gradient -->
      <radialGradient id="ontara-globe-grad" cx="62%" cy="40%" r="68%">
        <stop offset="0%" stop-color="#4a2d7a" />
        <stop offset="55%" stop-color="#24143d" />
        <stop offset="100%" stop-color="#0e081a" />
      </radialGradient>

      <!-- Nocturnal Amethyst Gradient for Crescent Inlay -->
      <linearGradient id="ontara-moon-inlay" x1="0%" y1="20%" x2="100%" y2="80%">
        <stop offset="0%" stop-color="#361c56" />
        <stop offset="50%" stop-color="#1f1033" />
        <stop offset="100%" stop-color="#2c1747" />
      </linearGradient>

      <!-- Rim Glow for Crescent Edge -->
      <linearGradient id="ontara-rim-glow" x1="10%" y1="0%" x2="80%" y2="100%">
        <stop offset="0%" stop-color="#fff8db" />
        <stop offset="50%" stop-color="#dfb76c" />
        <stop offset="100%" stop-color="#7a5016" />
      </linearGradient>

      <!-- Soft Celestial Glow Filter -->
      <filter id="ontara-star-glow" x="-30%" y="-30%" width="160%" height="160%">
        <feDropShadow dx="0" dy="0" stdDeviation="2.5" flood-color="#ffd88a" flood-opacity="0.8" />
      </filter>
    </defs>

    <!-- ===================================================================
         LAYER 1 (z-10): The Ornate Celestial Crescent Moon (Master Engraving)
         =================================================================== -->
    <g id="layer-crescent-moon">
      <!-- Outer Gilded Crescent Foundation with Beveled Rim -->
      <path
        d="M 45 4 C 22 8 4 28 4 52 C 4 76 22 96 56 96 C 38 88 22 72 22 52 C 22 32 32 16 45 4 Z"
        fill="url(#ontara-gold-grad)"
        stroke="url(#ontara-rim-glow)"
        stroke-width="1.2"
      />

      <!-- Inlaid Deep Amethyst Celestial Field -->
      <path
        d="M 43 8 C 24 12 7 30 7 52 C 7 73 24 92 52 93 C 36 85 24 70 24 52 C 24 35 34 20 43 8 Z"
        fill="url(#ontara-moon-inlay)"
      />

      <!-- Concentric Delicate Inner Gilded Thread Line -->
      <path
        d="M 41 12 C 26 16 10 32 10 52 C 10 71 25 88 48 90 C 35 83 26 69 26 52 C 26 37 34 23 41 12 Z"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.55"
        stroke-dasharray="2 1.5"
        opacity="0.65"
      />

      <!-- Astrolabe Graduation Ticks Along the Crescent Spine -->
      <path
        d="M 14 30 L 17 31 M 10 38 L 13 39 M 8 46 L 11 46.5 M 7.5 52 L 10.5 52 M 8 58 L 11 57.5 M 10 66 L 13 65 M 14 74 L 17 73 M 20 82 L 23 80 M 30 89 L 32 86"
        stroke="url(#ontara-rim-glow)"
        stroke-width="0.75"
        stroke-linecap="round"
        opacity="0.8"
      />

      <!-- Celestial Cartographic Scrollwork Flourish Inside the Spine -->
      <path
        d="M 13 43 C 12 48 15 52 19 51 C 15 50 14 46 15 42"
        stroke="url(#ontara-gold-grad)"
        stroke-width="0.85"
        fill="none"
        stroke-linecap="round"
        opacity="0.85"
      />
      <circle cx="15" cy="42" r="0.9" fill="#fff8db" />

      <!-- Inlaid Micro-Starlet on the Moon Flank -->
      <polygon
        points="12,52 13.2,50.8 14.4,52 13.2,53.2"
        fill="#ffffff"
        opacity="0.95"
      />

      <!-- Star Accent Beads Inset Along the Crescent -->
      <circle cx="21" cy="24" r="1" fill="url(#ontara-gold-grad)" />
      <circle cx="14" cy="61" r="1.1" fill="url(#ontara-gold-grad)" />
      <circle cx="23" cy="74" r="1.1" fill="url(#ontara-gold-grad)" />

      <!-- Crescent Horn Terminals (Gilded Spherical Finials) -->
      <circle cx="45" cy="4" r="1.6" fill="url(#ontara-gold-grad)" stroke="#fff8db" stroke-width="0.5" />
      <circle cx="56" cy="96" r="1.6" fill="url(#ontara-gold-grad)" stroke="#fff8db" stroke-width="0.5" />
    </g>

    <!-- ===================================================================
         LAYER 2 (z-20): The World / Earth Sphere (Deep Indigo & Meridians)
         =================================================================== -->
    <g id="layer-world-sphere">
      <!-- Base Globe Sphere -->
      <circle
        cx="56"
        cy="52"
        r="34"
        fill="url(#ontara-globe-grad)"
        stroke="url(#ontara-gold-grad)"
        stroke-width="1"
      />

      <!-- Continental Terrain Silhouette Hint (Imperial Purple) -->
      <path
        d="M40 38 Q46 32 54 36 Q62 40 58 48 Q52 56 46 62 Q38 68 34 58 Q34 46 40 38 Z"
        fill="#613b9c"
        opacity="0.35"
      />
      <path
        d="M62 58 Q72 54 78 62 Q82 72 74 78 Q66 82 62 74 Z"
        fill="#613b9c"
        opacity="0.3"
      />

      <!-- Meridian Curves (Longitude) -->
      <ellipse
        cx="56"
        cy="52"
        rx="15"
        ry="34"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.75"
        opacity="0.45"
      />
      <ellipse
        cx="56"
        cy="52"
        rx="26"
        ry="34"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.6"
        opacity="0.3"
      />

      <!-- Parallels of Latitude -->
      <ellipse
        cx="56"
        cy="37"
        rx="30"
        ry="10"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.65"
        opacity="0.35"
      />
      <line
        x1="22"
        y1="52"
        x2="90"
        y2="52"
        stroke="#dfb76c"
        stroke-width="0.8"
        stroke-dasharray="2 1.5"
        opacity="0.5"
      />
      <ellipse
        cx="56"
        cy="67"
        rx="30"
        ry="10"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.65"
        opacity="0.35"
      />

      <!-- Delicate Tilted Celestial Orbital Ring with Bead -->
      <ellipse
        cx="56"
        cy="52"
        rx="40"
        ry="13"
        transform="rotate(-22 56 52)"
        fill="none"
        stroke="#dfb76c"
        stroke-width="0.55"
        opacity="0.45"
        stroke-dasharray="2 2"
      />
      <circle cx="89" cy="40" r="1.8" fill="url(#ontara-gold-grad)" />
    </g>

    <!-- ===================================================================
         LAYER 3 (z-30): The Radiant Golden Compass Star (Top Centered Layer)
         =================================================================== -->
    <g id="layer-compass-star" filter="url(#ontara-star-glow)">
      <!-- Equatorial Axis Ray -->
      <line
        x1="38"
        y1="52"
        x2="86"
        y2="52"
        stroke="url(#ontara-rim-glow)"
        stroke-width="1"
        opacity="0.85"
      />

      <!-- 8-Pointed Celestial Compass Star Centered on Globe Core (62, 52) -->
      <!-- Cardinal North Ray -->
      <polygon points="62,52 60.5,43 62,32 63.5,43" fill="url(#ontara-star-grad)" />
      <!-- Cardinal South Ray -->
      <polygon points="62,52 60.5,61 62,72 63.5,61" fill="url(#ontara-star-grad)" />
      <!-- Cardinal East Ray -->
      <polygon points="62,52 71,50.5 82,52 71,53.5" fill="url(#ontara-star-grad)" />
      <!-- Cardinal West Ray -->
      <polygon points="62,52 53,50.5 42,52 53,53.5" fill="url(#ontara-star-grad)" />

      <!-- Diagonal Corner Rays (Shorter) -->
      <polygon points="62,52 64.5,46 70,44 66,49.5" fill="url(#ontara-star-grad)" opacity="0.9" />
      <polygon points="62,52 64.5,58 70,60 66,54.5" fill="url(#ontara-star-grad)" opacity="0.9" />
      <polygon points="62,52 59.5,46 54,44 58,49.5" fill="url(#ontara-star-grad)" opacity="0.9" />
      <polygon points="62,52 59.5,58 54,60 58,54.5" fill="url(#ontara-star-grad)" opacity="0.9" />

      <!-- Astrolabe Center Ring & Core Jewel -->
      <circle
        cx="62"
        cy="52"
        r="3.2"
        fill="#24143d"
        stroke="url(#ontara-rim-glow)"
        stroke-width="1.2"
      />
      <circle cx="62" cy="52" r="1.4" fill="#ffffff" />
    </g>
  </svg>
</div>
