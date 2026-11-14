export const ssr = true;
export const prerender = true;
export const trailingSlash = 'always';
export function entries() { return [{lang: ''}, ...['de','fa','ru'].map(lang => ({lang}))]; }
