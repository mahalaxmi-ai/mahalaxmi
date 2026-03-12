'use client';

import { Box, CircularProgress } from '@mui/material';

const LoadingSpinner = ({ size = 40, color = 'primary', sx = {} }) => {
  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        p: 2,
        ...sx,
      }}
    >
      <CircularProgress size={size} color={color} />
    </Box>
  );
};

export default LoadingSpinner;
