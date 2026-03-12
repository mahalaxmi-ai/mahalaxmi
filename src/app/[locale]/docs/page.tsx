import type { Metadata } from 'next';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardActionArea from '@mui/material/CardActionArea';
import CardContent from '@mui/material/CardContent';
import Container from '@mui/material/Container';
import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Mahalaxmi AI Documentation',
  description: 'Everything you need to get Mahalaxmi AI running',
};

interface DocCard {
  title: string;
  description: string;
  href: string;
}

const docSections: DocCard[] = [
  {
    title: 'Quickstart',
    description: 'Install and run your first AI orchestration cycle',
    href: '/docs/quickstart',
  },
  {
    title: 'VS Code Extension',
    description: 'Connect VS Code to your cloud server',
    href: '/docs/vscode',
  },
  {
    title: 'Cloud Servers',
    description: 'Set up and manage cloud orchestration servers',
    href: '/docs/cloud',
  },
  {
    title: 'API Reference',
    description: 'Headless API for power users',
    href: '/docs/api',
  },
  {
    title: 'FAQ',
    description: 'Common questions answered',
    href: '/docs/faq',
  },
];

export default function DocsPage() {
  return (
    <Container maxWidth="lg" sx={{ py: 8 }}>
      <Box sx={{ mb: 6 }}>
        <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
          Mahalaxmi AI Documentation
        </Typography>
        <Typography variant="h6" color="text.secondary">
          Everything you need to get Mahalaxmi AI running
        </Typography>
      </Box>

      <Grid container spacing={3}>
        {docSections.map((section) => (
          <Grid item xs={12} sm={6} md={4} key={section.href}>
            <Card
              sx={{
                height: '100%',
                bgcolor: 'background.paper',
                border: '1px solid',
                borderColor: 'divider',
                transition: 'border-color 0.2s',
                '&:hover': {
                  borderColor: 'primary.main',
                },
              }}
            >
              <CardActionArea
                component={Link}
                href={section.href}
                sx={{ height: '100%' }}
              >
                <CardContent sx={{ p: 3 }}>
                  <Typography variant="h6" component="h2" gutterBottom fontWeight={600}>
                    {section.title}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {section.description}
                  </Typography>
                </CardContent>
              </CardActionArea>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Container>
  );
}
