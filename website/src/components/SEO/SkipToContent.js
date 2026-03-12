'use client';

import { Box } from '@mui/material';
import { useTranslations } from 'next-intl';

const SkipToContent = () => {
  const t = useTranslations('common');

  return (
    <Box
      component="a"
      href="#main-content"
      sx={{
        position: 'absolute',
        left: '-9999px',
        top: 'auto',
        width: '1px',
        height: '1px',
        overflow: 'hidden',
        zIndex: 9999,
        '&:focus': {
          position: 'fixed',
          top: 8,
          left: 8,
          width: 'auto',
          height: 'auto',
          overflow: 'visible',
          bgcolor: 'primary.main',
          color: 'primary.contrastText',
          px: 3,
          py: 1.5,
          borderRadius: 1,
          fontSize: '0.875rem',
          fontWeight: 600,
          textDecoration: 'none',
          boxShadow: 4,
        },
      }}
    >
      {t('skipToContent')}
    </Box>
  );
};

export default SkipToContent;
