export const save = async (image, format) => {
  return import('../pkg').then(converter => converter.convert(image, format));
};
