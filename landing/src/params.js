export const params = {
  locale: {
    '~standard': {
      version: 1,
      vendor: 'sigma',
      validate: value => ['de', 'fa', 'ru'].includes(value)
        ? { value }
        : { issues: [{ message: 'Unsupported language' }] }
    }
  }
};
