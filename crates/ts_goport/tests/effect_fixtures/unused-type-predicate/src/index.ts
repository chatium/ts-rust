export const isString = (value: string | number): value is string => true;

export function isNumber(input: unknown): input is number {
  return true;
}
