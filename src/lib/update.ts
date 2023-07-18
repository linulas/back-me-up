import { checkUpdate, installUpdate } from '@tauri-apps/api/updater';
import { relaunch } from '@tauri-apps/api/process';
import { info, error } from 'tauri-plugin-log-api';

type UpdateProgress = 'already-updated' | 'available-but-aborted' | 'installed' | 'error';
interface UpdateInfo {
	status: UpdateProgress;
	version?: string;
	date?: string;
	body?: string;
	message?: string;
}

export const checkForUpdate = async (): Promise<UpdateInfo> => {
	try {
    info('Checking for update...');

		const { shouldUpdate, manifest } = await checkUpdate();

		info(JSON.stringify({ shouldUpdate, manifest }));

		if (!shouldUpdate) return { status: 'already-updated' };

		// HACK: Must type confirm as any because typescript doesn't type it as a promise
		const updateConfirmed: Promise<boolean> = await (confirm as any)(
			`Update downloaded, would you like to install?`
		);
		if (!updateConfirmed) return { status: 'available-but-aborted' };

		info(`Installing update ${manifest?.version}, ${manifest?.date}, ${manifest?.body}`);
		await installUpdate();

		const confirmRestart: Promise<boolean> = await (confirm as any)(
			'A restart is required for the update to take effect\nWould you like to restart now?'
		);

		const update: UpdateInfo = {
			status: 'installed',
			version: manifest?.version,
			date: manifest?.date,
			body: manifest?.body
		};

		if (!confirmRestart) return update;
		await relaunch();

		return update; // just to make TS happy
	} catch (e) {
		const message = `Error checking for update: ${e}`;
		error(message);
		return { status: 'error', message };
	}
};
