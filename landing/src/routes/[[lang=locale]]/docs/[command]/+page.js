import { error } from '@sveltejs/kit';
import { commands } from '#lib/commands.js';

export function entries() {
  return ['', 'de', 'fa', 'ru'].flatMap(lang => commands.map(({ name }) => ({ lang, command: name })));
}

export function load({ params }) {
  const command = commands.find(({ name }) => name === params.command);
  if (!command) error(404, 'Unknown sigma command');
  return { command };
}
