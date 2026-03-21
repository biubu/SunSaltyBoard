import { formatDistanceToNow } from "date-fns";

export function formatTimeAgo(dateString: string): string {
  try {
    const date = new Date(dateString);
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

export function cn(...classes: (string | boolean | undefined | null)[]): string {
  return classes.filter(Boolean).join(" ");
}
