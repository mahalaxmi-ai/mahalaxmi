import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';

export const metadata = {
  title: 'Contributing to Mahalaxmi',
  description: 'Learn how to contribute to Mahalaxmi open source — CLA requirement, accepted contributions, PR process, and code of conduct.',
  alternates: {
    canonical: '/open-source/contributing',
  },
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const steps = [
  'Fork the repository on GitHub.',
  'Create a feature branch: git checkout -b feat/your-feature-name',
  'Implement your changes and write tests for any new code paths.',
  'Run the full test suite and ensure it passes.',
  'Open a Pull Request targeting the main branch with a clear description of the change.',
];

const acceptedContributions = [
  'Bug fixes — reproducible bugs with a test case demonstrating the fix.',
  'Provider plugins — new AI CLI adapters implementing the Provider Plugin SDK interface.',
  'Documentation — corrections, clarifications, and additions to existing docs.',
  'Minor improvements — performance fixes, code cleanup, and dependency updates.',
];

const requiresDiscussion = [
  'Core orchestration engine changes — open an issue first to discuss the design.',
  'New consensus or merge strategies — architectural discussion required before implementation.',
  'Breaking API changes — requires maintainer sign-off on the migration path.',
];

export default function ContributingPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Contributing to Mahalaxmi
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Mahalaxmi is open source and contributions are welcome. Please read the guidelines below
        before submitting a Pull Request.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* CLA Requirement */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Contributor License Agreement (CLA)
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        All contributors must sign a{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontWeight: 600 }}>
          Contributor License Agreement (CLA)
        </Box>{' '}
        before any Pull Request can be accepted. The CLA ensures that ThriveTech Services LLC
        holds the necessary rights to distribute your contributions as part of the project,
        and that you retain copyright over your own work.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The CLA bot will prompt you to sign when you open your first PR. You only need to sign
        once — it covers all future contributions to the Mahalaxmi repository.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Accepted Contributions */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        What Contributions Are Accepted
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1.5 }}>
        The following contributions are accepted without prior discussion:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        {acceptedContributions.map((item) => (
          <Typography key={item} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            {item}
          </Typography>
        ))}
      </Box>

      <Typography variant="body1" color="text.secondary" sx={{ mb: 1.5 }}>
        The following require an issue or discussion before you start work:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        {requiresDiscussion.map((item) => (
          <Typography key={item} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            {item}
          </Typography>
        ))}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* How to Submit a PR */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        How to Submit a Pull Request
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 3 }}>
        {steps.map((step, index) => (
          <Typography key={index} component="li" variant="body1" color="text.secondary" sx={{ mb: 1.5 }}>
            {step}
          </Typography>
        ))}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Code of Conduct */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Code of Conduct
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Mahalaxmi is committed to providing a welcoming and respectful environment for all
        contributors. We expect all participants to:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        {[
          'Use welcoming and inclusive language.',
          'Respect differing viewpoints and experiences.',
          'Accept constructive criticism gracefully.',
          'Focus on what is best for the community and the project.',
          'Show empathy toward other community members.',
        ].map((item) => (
          <Typography key={item} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            {item}
          </Typography>
        ))}
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Harassment, personal attacks, and discriminatory behavior of any kind will not be
        tolerated. Violations may result in removal from the project.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* CONTRIBUTING.md link */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Full Guidelines
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        See CONTRIBUTING.md on GitHub for the complete contribution guidelines, including
        commit message conventions, testing requirements, and release process.
      </Typography>

      <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi/blob/main/CONTRIBUTING.md"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-block',
            color: '#00C8C8',
            border: '1px solid #00C8C8',
            borderRadius: 1,
            px: 2,
            py: 1,
            textDecoration: 'none',
            fontWeight: 600,
            '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
          }}
        >
          View CONTRIBUTING.md on GitHub →
        </Box>
      </Box>

      <Divider sx={{ my: 4 }} />

      <Typography variant="body2" color="text.secondary">
        Questions about contributing?{' '}
        <Box
          component="a"
          href="mailto:support@mahalaxmi.ai"
          sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          support@mahalaxmi.ai
        </Box>
      </Typography>
    </Container>
  );
}
