export const extractFileNameFromPath = (path: string) => {
  const parts = path.split('/');
  return parts[parts.length - 1];
}
