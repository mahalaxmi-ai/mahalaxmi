'use client';

import { useState } from 'react';
import { IconButton, Menu, MenuItem, ListItemText, Tooltip } from '@mui/material';
import LanguageIcon from '@mui/icons-material/Language';
import { useRouter, usePathname } from 'next-intl/client';
import { useLocale } from 'next-intl';

const locales = [
  { code: 'en-US', label: 'English', display: 'EN' },
  { code: 'es-ES', label: 'Español', display: 'ES' },
  { code: 'fr-FR', label: 'Français', display: 'FR' },
  { code: 'de-DE', label: 'Deutsch', display: 'DE' },
  { code: 'pt-BR', label: 'Português', display: 'PT' },
  { code: 'ja-JP', label: '日本語', display: 'JA' },
  { code: 'zh-CN', label: '中文', display: 'ZH' },
  { code: 'ko-KR', label: '한국어', display: 'KO' },
  { code: 'hi-IN', label: 'हिन्दी', display: 'HI' },
  { code: 'ar-SA', label: 'العربية', display: 'AR' },
];

const LanguageSelector = () => {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const router = useRouter();
  const pathname = usePathname();
  const currentLocale = useLocale();

  const currentLocaleData = locales.find((l) => l.code === currentLocale) ?? locales[0];

  const handleOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleSelect = (localeCode: string) => {
    router.replace(pathname, { locale: localeCode });
    handleClose();
  };

  return (
    <>
      <Tooltip title="Select language">
        <IconButton
          onClick={handleOpen}
          color="inherit"
          aria-label="select language"
          aria-haspopup="true"
          aria-expanded={Boolean(anchorEl)}
          size="small"
          sx={{ gap: 0.5, fontSize: '0.75rem', fontWeight: 700 }}
        >
          <LanguageIcon fontSize="small" />
          {currentLocaleData.display}
        </IconButton>
      </Tooltip>
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleClose}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        transformOrigin={{ vertical: 'top', horizontal: 'right' }}
      >
        {locales.map((locale) => (
          <MenuItem
            key={locale.code}
            selected={locale.code === currentLocale}
            onClick={() => handleSelect(locale.code)}
          >
            <ListItemText
              primary={locale.label}
              secondary={locale.display}
            />
          </MenuItem>
        ))}
      </Menu>
    </>
  );
};

export default LanguageSelector;
