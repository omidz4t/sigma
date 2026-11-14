import { resolve as kitResolve } from '$app/paths';
const base = kitResolve('/').replace(/\/$/, '');
import en from './locales/en.json';
import de from './locales/de.json';
import fa from './locales/fa.json';
import ru from './locales/ru.json';

export const languages = [
  { code: 'en', label: 'English' },
  { code: 'de', label: 'Deutsch' },
  { code: 'fa', label: 'فارسی' },
  { code: 'ru', label: 'Русский' }
];
export const language = $state({ current: 'en' });
const dictionaries = { en, de, fa, ru };
export function t(text) {
  return dictionaries[language.current]?.[text] || text;
}
export function browserLanguage(preferences) {
  for (const preference of preferences) {
    const code = preference.toLowerCase().split(/[-_]/)[0];
    if (dictionaries[code]) return code;
  }
  return 'en';
}
export function chooseLanguage(code, persist = true) {
  if (!dictionaries[code]) return;
  language.current = code;
  if (persist) {
    try { localStorage.setItem('sigma-language', code); } catch { /* Storage may be blocked. */ }
  }
}
export function initializeLanguage() {
  let saved;
  try { saved = localStorage.getItem('sigma-language'); } catch { /* Use browser preferences. */ }
  chooseLanguage(dictionaries[saved] ? saved : browserLanguage(navigator.languages || [navigator.language]), false);
}

export function localizedPath(pathname, code = language.current) {
  const path = base && (pathname === base || pathname.startsWith(base + '/')) ? pathname.slice(base.length) : pathname;
  const unprefixed = path.replace(/^\/(de|fa|ru)(?=\/|$)/, '') || '/';
  return `${base}${code === 'en' ? '' : '/' + code}${unprefixed.startsWith('/') ? unprefixed : '/' + unprefixed}`;
}
export function resolve(path, params = {}) {
  for (const [key, value] of Object.entries(params)) path = path.replace(`[${key}]`, encodeURIComponent(value));
  return localizedPath(path);
}
