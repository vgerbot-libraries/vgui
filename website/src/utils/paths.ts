function trimSlashes(value: string): string {
  return value.replace(/^\/+|\/+$/g, '');
}

/** Build an internal URL that works both locally and under the GitHub Pages base. */
export function basePath(path = ''): string {
  const base = import.meta.env.BASE_URL.replace(/\/$/, '');
  const suffix = trimSlashes(path);
  const trailingSlash = suffix && !suffix.split('/').pop()?.includes('.') ? '/' : '';
  return `${base}/${suffix}${trailingSlash}`;
}

/** Build an internal URL for the active Starlight locale. */
export function localizedPath(locale: string | null | undefined, path = ''): string {
  const localePrefix = locale ? trimSlashes(locale) : '';
  const suffix = trimSlashes(path);
  return basePath([localePrefix, suffix].filter(Boolean).join('/'));
}
