import { formatDistanceToNow } from "date-fns";

export function formatTimeAgo(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHr = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHr / 24);

    if (diffSec < 60) return `${diffSec}s`;
    if (diffMin < 60) return `${diffMin}m`;
    if (diffHr < 24) return `${diffHr}h`;
    if (diffDay < 7) return `${diffDay}d`;
    return formatDistanceToNow(date, { addSuffix: true });
  } catch {
    return dateString;
  }
}

export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + "...";
}

export function getContentTypeIcon(contentType: string): string {
  switch (contentType) {
    case "text":
      return "📝";
    case "image":
      return "🖼️";
    case "file":
      return "📁";
    case "html":
      return "🌐";
    case "rtf":
      return "📄";
    default:
      return "📋";
  }
}

export function getContentTypeColor(contentType: string): string {
  switch (contentType) {
    case "text":
      return "text-blue-400";
    case "image":
      return "text-purple-400";
    case "file":
      return "text-yellow-400";
    case "html":
      return "text-green-400";
    case "rtf":
      return "text-orange-400";
    default:
      return "text-gray-400";
  }
}


