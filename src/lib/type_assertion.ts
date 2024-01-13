export const isBackupFolderJob = (job: App.Job): job is App.BackupFolderJob => {
	return (job as any)?.from !== undefined && (job as any)?.to !== undefined;
};

export const isOneTimeBackupJob = (job: App.Job): job is App.BackupFolderJob => {
	return (
		isBackupFolderJob(job) && job.__frequency === 'one-time'
	);
};

export const isAppError = (entity: any): entity is App.Error => {
  return entity?.message !== undefined;
}
