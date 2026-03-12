import Image from 'next/image';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Divider from '@mui/material/Divider';
import Link from '@mui/material/Link';

export const metadata = {
  title: 'About — Mahalaxmi',
  alternates: {
    canonical: '/about',
  },
};

const ACCENT = '#00C8C8';
const BG = '#0A2A2A';

function SectionHeading({ children }) {
  return (
    <Typography
      variant="h4"
      component="h2"
      sx={{ color: ACCENT, fontWeight: 700, mb: 2, mt: 6 }}
    >
      {children}
    </Typography>
  );
}

export default function AboutPage() {
  return (
    <Box sx={{ bgcolor: BG, minHeight: '100vh', color: '#E0F7FA', py: 0 }}>
      {/* Hero */}
      <Box
        sx={{
          bgcolor: '#061A1A',
          py: { xs: 8, md: 12 },
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          textAlign: 'center',
          px: 2,
        }}
      >
        <Box sx={{ mb: 4 }}>
          <Image
            src="/mahalaxmi_logo.png"
            alt="Mahalaxmi logo"
            width={120}
            height={120}
            priority
          />
        </Box>
        <Typography
          variant="h2"
          component="h1"
          sx={{ fontWeight: 800, color: ACCENT, mb: 2, fontSize: { xs: '2rem', md: '3rem' } }}
        >
          About Mahalaxmi
        </Typography>
        <Typography variant="h6" sx={{ color: '#B2DFDB', maxWidth: 640 }}>
          AI orchestration tooling for engineering teams — built by ThriveTech Services LLC.
        </Typography>
      </Box>

      <Container maxWidth="md" sx={{ pb: 12 }}>
        {/* Company */}
        <SectionHeading>Company</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          Mahalaxmi is a product of{' '}
          <strong>ThriveTech Services LLC</strong>, headquartered in the United States. ThriveTech
          builds AI orchestration tooling that helps engineering teams move faster by running
          multiple AI coding agents in parallel — cutting through complexity so teams can focus on
          shipping.
        </Typography>

        <Divider sx={{ borderColor: '#1E4040', my: 4 }} />

        {/* Product story */}
        <SectionHeading>The Story</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8, mb: 3 }}>
          Mahalaxmi started as an internal tool: a way to run multiple Claude Code agents in
          parallel across a single codebase. The idea was simple — give each agent a dedicated task
          and let them work simultaneously, then merge the results. What began as a productivity
          experiment quickly became something the team couldn't live without.
        </Typography>
        <Typography variant="body1" sx={{ lineHeight: 1.8, mb: 3 }}>
          The project's first iteration was called <strong>Ganesha</strong> — named after the Hindu
          deity renowned as the remover of obstacles. Ganesha cleared the path: a lean orchestrator
          that dispatched agents and collected their output. As the platform matured and ambitions
          grew, the project was reborn as <strong>Mahalaxmi</strong>.
        </Typography>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          In Hindu tradition, Mahalaxmi is the goddess of prosperity, success, and abundance. The
          name reflects the platform's promise: not just removing obstacles, but multiplying what
          engineering teams can achieve. From Ganesha to Mahalaxmi — from clearing the path to
          accelerating the journey.
        </Typography>

        <Divider sx={{ borderColor: '#1E4040', my: 4 }} />

        {/* Mission */}
        <SectionHeading>Mission</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          Make multi-agent AI orchestration accessible to every engineering team. Whether you're a
          solo developer or a large engineering org, Mahalaxmi gives you the infrastructure to run
          parallel AI workflows without the overhead of building and maintaining that infrastructure
          yourself.
        </Typography>

        <Divider sx={{ borderColor: '#1E4040', my: 4 }} />

        {/* Open Source */}
        <SectionHeading>Open Source</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          Mahalaxmi is developed in the open. You can follow along, contribute, or fork the project
          on GitHub:
        </Typography>
        <Box sx={{ mt: 2 }}>
          <Link
            href="https://github.com/thrivetech2t/mahalaxmi"
            target="_blank"
            rel="noopener noreferrer"
            sx={{ color: ACCENT, fontWeight: 600, fontSize: '1rem' }}
          >
            github.com/thrivetech2t/mahalaxmi
          </Link>
        </Box>

        <Divider sx={{ borderColor: '#1E4040', my: 4 }} />

        {/* Team */}
        <SectionHeading>The Team</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          Mahalaxmi is built and maintained by the team at{' '}
          <strong>ThriveTech Services LLC</strong>, headquartered in the United States. We're a
          small, focused team obsessed with developer productivity and the practical applications of
          AI in software engineering workflows.
        </Typography>

        <Divider sx={{ borderColor: '#1E4040', my: 4 }} />

        {/* Contact */}
        <SectionHeading>Contact</SectionHeading>
        <Typography variant="body1" sx={{ lineHeight: 1.8, mb: 1 }}>
          <strong>General inquiries:</strong>{' '}
          <Link href="mailto:support@mahalaxmi.ai" sx={{ color: ACCENT }}>
            support@mahalaxmi.ai
          </Link>
        </Typography>
        <Typography variant="body1" sx={{ lineHeight: 1.8 }}>
          <strong>Enterprise &amp; sales:</strong>{' '}
          <Link href="mailto:sales@mahalaxmi.ai" sx={{ color: ACCENT }}>
            sales@mahalaxmi.ai
          </Link>
        </Typography>
      </Container>
    </Box>
  );
}
