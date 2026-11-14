const initial = { image: '01-initial.png', caption: 'The Bot Master’s Delta Chat account, with its custom profile photo. The QR button is at the top right.' };
export const terminalSteps = [
    {
      title: 'Create the bot account.',
      frames: [{ ...initial, description: 'This creates a separate bot account on your machine. It does not connect the bot to you yet. Keep Delta Chat open on your phone.', command: './sigma init' }]
    },
    {
      title: 'Connect with your phone.',
      frames: [
        { ...initial, pointer: [82.2, 6.8], description: 'In Delta Chat, tap the QR button at the top right. Open QR Invite Code to find your personal invitation.' },
        { image: '02-qr.png', pointer: [34, 73.5], caption: 'The actual QR Invite Code screen. Tap LINK below the QR code.', description: 'The actual QR Invite Code screen. Tap LINK below the QR code.' },
        { image: '03-copy-link.png', pointer: [67, 79.8], caption: 'Tap COPY TO CLIPBOARD, then use the copied Delta Chat link in your terminal.', description: 'Tap LINK, then COPY TO CLIPBOARD. Paste that invitation into the terminal command in the next step. init alone never links the accounts.', command: './sigma create-contact me "https://i.delta.chat/#DEMO..."' },
        { image: '04-contact.png', caption: 'Secure join completed by create-contact. Sigma now appears as a bot in Delta Chat.', description: 'Replace the quoted placeholder with the link you copied. create-contact completes secure join and saves the alias me. This is the step that connects you to the bot.', command: './sigma create-contact me "https://i.delta.chat/#DEMO..."' }
      ]
    },
    {
      title: 'Send a message and a file.',
      frames: [
        { image: '05-message.png', caption: 'The real message, sent by Sigma and received in the Android app.', description: 'Send to the saved alias. Your message arrives in the encrypted conversation with Sigma.', command: './sigma send-message me "Hello from the terminal! This message was sent with Sigma."' },
        { image: '06-file.png', caption: 'The report.txt attachment delivered by send-file.', description: 'Send a local report, backup, or build artifact to the same conversation.', command: './sigma send-file me ./report.txt' }
      ]
    }
  ];
