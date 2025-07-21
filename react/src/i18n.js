import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// Translation files
import enTranslation from './locales/en/translation.json';
import koTranslation from './locales/ko/translation.json';

const resources = {
  en: {
    translation: enTranslation,
  },
  ko: {
    translation: koTranslation,
  },
};

i18n
  .use(initReactI18next) // passes i18n down to react-i18next
  .init({
    resources,
    lng: 'ko', // default language
    fallbackLng: 'en', // fallback language if translation is not found

    interpolation: {
      escapeValue: false, // react already escapes by default
    },
  });

export default i18n;
