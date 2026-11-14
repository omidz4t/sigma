import { t } from './i18n.svelte.js';
import { commands, commandGroups } from './commands.js';

export function commandMarkdown(command) {
  return `# ${command.name}\n\n${t(command.description)}\n\n## ${t("Usage")}\n\n\`\`\`sh\n${command.usage}\n\`\`\`\n\n## ${t("Options")}\n\n\`\`\`text\n${command.options}\n\`\`\`\n\n## ${t("Example")}\n\n\`\`\`sh\n${command.example}\n\`\`\`\n\nOptions may appear before or after arguments. Quote text, paths with spaces, and invitation links. Use -- before literal arguments beginning with -.\n\nAccount data defaults to ~/.config/sigma; use the same --data-path for the same account. Run ./sigma ${command.name} --help to verify your installed version.\n`;
}

export function pageMarkdown(name) {
  const command = commands.find(command => command.name === name);
  if (command) return commandMarkdown(command);
  return '# Sigma command reference\n\nUsage, options, and examples for all sigma commands. Account state defaults to ~/.config/sigma.\n\n' + commandGroups.map(group => `## ${t(group.title)}\n\n` + group.commands.map(command => `### ${command.name}\n\n${t(command.description)}\n\nUsage: ${command.usage}\n`).join('\n')).join('\n');
}
