// src/utils/formatTime.js
import { useTranslation } from 'react-i18next';

export default function formatTime(dateString) {
    const { t } = useTranslation();
    const date = new Date(dateString);
    const now = new Date();
    const msPerDay = 24 * 60 * 60 * 1000;
    const diffMs = now - date;
    const diffDays = Math.floor(diffMs / msPerDay);
    if (diffDays < 1) return t('today');
    if (diffDays === 1) return t('one_day_ago');
    if (diffDays === 2) return t('two_days_ago');
    if (diffDays < 7) return t('days_ago', { count: diffDays });
    return date.toLocaleDateString();
  }
  