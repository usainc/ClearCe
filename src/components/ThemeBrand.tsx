const themes = ["dark", "light", "midnight", "graphite", "forest"] as const;

/** CSS follows the resolved theme, including live Windows System changes. */
export function ThemeBrand({ symbol = false, className = "", decorative = false }: {
  symbol?: boolean; className?: string; decorative?: boolean;
}) {
  return <span className={`theme-brand ${className}`} role={decorative ? undefined : "img"}
    aria-label={decorative ? undefined : "ClearCe — Local AI Image Enhancer — by UsainCe.dev"} aria-hidden={decorative || undefined}>
    {themes.map(theme => <img key={theme} data-brand-theme={theme} alt="" draggable={false}
      width={symbol ? 128 : 600} height={symbol ? 128 : 525}
      src={`/brand/themes/${theme}-${symbol ? "symbol" : "logo"}.png`} />)}
  </span>;
}
