/**
 * Standard date, time, and duration formatting for Dicta.
 */

export function formatTime(isoDate?: string | null): string {
  if (!isoDate) return '--:--';
  const d = new Date(isoDate);
  if (Number.isNaN(d.getTime())) return '--:--';
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

export function formatDateHeader(date: Date, isRu = false): string {
  const day = date.getDate();
  const month = date.toLocaleDateString(isRu ? 'ru-RU' : 'en-US', { month: 'long' });
  return `${day} ${month}`;
}

export function formatDuration(sec?: number): string {
  if (!sec || sec <= 0) return '00:00';
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}
