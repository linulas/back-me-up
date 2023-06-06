export const isUrl = (str: string): boolean => {
	const urlRegex = /^(?:(?:https?|ftp):\/\/|www\.)[^\s/$.?#].[^\s]*$/i;
	return urlRegex.test(str);
};

export const isNumber = (value: any): boolean => {
	return typeof value === 'number' && isFinite(value);
};
