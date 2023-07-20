export const isUrl = (str: string): boolean => {
	const urlRegex = /^(?:(?:https?|ftp):\/\/|www\.)[^\s/$.?#].[^\s]*$/i;
	return urlRegex.test(str);
};

export const isIp = (str: string): boolean => {
	const reg =
		/^(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])\.(\d{1,2}|1\d\d|2[0-4]\d|25[0-5])$/;
	return reg.test(str);
};

export const isNumber = (value: any): boolean => {
	return typeof value === 'number' && isFinite(value);
};

export type Field = { name: string; defaultValue: any };
export const missingKeys = (obj: object, fields: Field[]): Field[] => {
	const missingKeys: Field[] = [];
	for (const field of fields) {
		if (!(field.name in obj)) {
			missingKeys.push(field);
		}
	}

	return missingKeys;
};
