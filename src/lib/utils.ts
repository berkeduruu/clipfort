export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diffSec = Math.floor((now - timestamp) / 1000);

  if (diffSec < 5) return 'Just now';
  if (diffSec < 60) return `${diffSec}s ago`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour}h ago`;
  const diffDay = Math.floor(diffHour / 24);
  if (diffDay === 1) return 'Yesterday';
  if (diffDay < 7) return `${diffDay}d ago`;
  
  const d = new Date(timestamp);
  return `${d.toLocaleDateString('en-US')} ${d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })}`;
}

export function formatBytes(bytes?: number): string {
  if (!bytes || bytes === 0) return '';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function parseColorCode(text: string): { color: string; hex: string } | null {
  if (!text) return null;
  const trimmed = text.trim();
  if (trimmed.length > 50) return null;

  // 1. Hex codes: #fff, #ffffff, #ffffffff
  const hexMatch = trimmed.match(/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/);
  if (hexMatch) {
    let hex = trimmed;
    if (trimmed.length === 4) {
      hex = `#${trimmed[1]}${trimmed[1]}${trimmed[2]}${trimmed[2]}${trimmed[3]}${trimmed[3]}`;
    } else if (trimmed.length === 5) {
      hex = `#${trimmed[1]}${trimmed[1]}${trimmed[2]}${trimmed[2]}${trimmed[3]}${trimmed[3]}`;
    } else if (trimmed.length === 9) {
      hex = trimmed.substring(0, 7);
    }
    return { color: trimmed, hex: hex.substring(0, 7) };
  }

  // 2. rgb / rgba
  const rgbMatch = trimmed.match(/^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})(?:\s*,\s*([\d.]+))?\s*\)$/i);
  if (rgbMatch) {
    const r = Math.min(255, parseInt(rgbMatch[1], 10));
    const g = Math.min(255, parseInt(rgbMatch[2], 10));
    const b = Math.min(255, parseInt(rgbMatch[3], 10));
    const toHex = (n: number) => n.toString(16).padStart(2, '0');
    return { color: trimmed, hex: `#${toHex(r)}${toHex(g)}${toHex(b)}` };
  }

  // 3. hsl / hsla
  const hslMatch = trimmed.match(/^hsla?\(\s*([\d.]+)\s*,\s*([\d.]+)%?\s*,\s*([\d.]+)%?(?:\s*,\s*([\d.]+))?\s*\)$/i);
  if (hslMatch) {
    return { color: trimmed, hex: '#10b981' };
  }

  return null;
}

export function parseUrl(text: string): { url: string; domain: string } | null {
  if (!text) return null;
  const trimmed = text.trim();
  if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) {
    try {
      const u = new URL(trimmed);
      return { url: trimmed, domain: u.hostname.replace(/^www\./, '') };
    } catch {
      return null;
    }
  }
  return null;
}
