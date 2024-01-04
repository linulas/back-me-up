export const isBackupFolderJob = (job: App.Job): job is App.BackupFolderJob => {
	return (job as any).from !== undefined && (job as any).to !== undefined;
};

export const isSingleBackupJob = (job: App.Job): job is App.BackupFolderJob => {
	return (
		isSingleBackupJob(job) && job.__type === 'single'
	);
};
