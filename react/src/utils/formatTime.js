// src/utils/formatTime.js
export default function formatTime(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const msPerDay = 24 * 60 * 60 * 1000;
    const diffMs = now - date;
    const diffDays = Math.floor(diffMs / msPerDay);
    if (diffDays < 1) return '오늘';
    if (diffDays === 1) return '하루 전';
    if (diffDays === 2) return '이틀 전';
    if (diffDays < 7) return `${diffDays}일 전`;
    return date.toLocaleDateString();
  }
  