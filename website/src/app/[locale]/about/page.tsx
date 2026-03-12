import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import Chip from '@mui/material/Chip';

export const metadata = {
  title: 'About Mahalaxmi AI',
  description: 'Learn about Mahalaxmi AI, built by ThriveTech Services LLC.',
};

const stackItems = ['Next.js', 'MUI', 'Docker', 'Hetzner Cloud'];

export default function AboutPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        About Mahalaxmi AI
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1 }}>
        Built by <strong>ThriveTech Services LLC</strong>
      </Typography>

      <Divider sx={{ my: 4 }} />

      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={600} gutterBottom>
          Our Story
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ lineHeight: 1.8 }}>
          Ganesha was our internal AI orchestration engine, born from the need to coordinate
          multiple AI coding agents on complex engineering tasks. Mahalaxmi is Ganesha
          productized — available to developers and teams worldwide.
        </Typography>
      </Box>

      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={600} gutterBottom>
          Mission
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ lineHeight: 1.8 }}>
          To make multi-agent AI orchestration accessible to every developer.
        </Typography>
      </Box>

      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={600} gutterBottom>
          Stack
        </Typography>
        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1, mt: 1 }}>
          {stackItems.map((item) => (
            <Chip key={item} label={item} variant="outlined" />
          ))}
        </Box>
      </Box>

      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={600} gutterBottom>
          Team
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Meet the team — coming soon.
        </Typography>
      </Box>
    </Container>
  );
}
