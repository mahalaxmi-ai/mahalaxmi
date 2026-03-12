import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import Chip from "@mui/material/Chip";
import Divider from "@mui/material/Divider";
import Alert from "@mui/material/Alert";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Paper from "@mui/material/Paper";

export const metadata = {
  title: "Cloud — Mahalaxmi Docs",
  description:
    "Complete guide to Mahalaxmi Cloud: setup, project naming, server lifecycle states, idle timeout, and managing your cloud server.",
};

const LIFECYCLE_STATES = [
  {
    state: "pending_payment",
    color: "warning",
    label: "Pending Payment",
    description:
      "The subscription checkout has been initiated but payment has not yet been confirmed. No infrastructure is provisioned at this stage. Once Stripe confirms the payment, the server automatically advances to provisioning.",
  },
  {
    state: "provisioning",
    color: "info",
    label: "Provisioning",
    description:
      "Payment is confirmed and Hetzner is building your virtual machine. This typically takes 1–3 minutes. The server is not yet reachable during this phase.",
  },
  {
    state: "active",
    color: "success",
    label: "Active",
    description:
      "The VM is running and your Mahalaxmi orchestration environment is fully operational. Workers can accept tasks and the dashboard reflects live metrics.",
  },
  {
    state: "degraded",
    color: "warning",
    label: "Degraded",
    description:
      "The VM is running but one or more health checks are failing. Your environment may be partially available. Investigate logs in the dashboard; the server will recover automatically if the condition clears.",
  },
  {
    state: "stopping",
    color: "default",
    label: "Stopping",
    description:
      "A stop operation has been requested. Active workers are draining and the VM shutdown sequence is in progress. No new tasks should be submitted during this transition.",
  },
  {
    state: "stopped",
    color: "default",
    label: "Stopped",
    description:
      "The VM has been destroyed to eliminate compute costs. Your subscription remains active and all project configuration is preserved. The server can be restarted at any time from the dashboard.",
  },
  {
    state: "deleting",
    color: "error",
    label: "Deleting",
    description:
      "A permanent delete has been confirmed. All resources — VM, storage, and associated data — are being removed. This operation cannot be cancelled once started.",
  },
  {
    state: "deleted",
    color: "error",
    label: "Deleted",
    description:
      "All server resources have been permanently destroyed. The project record is retained for billing history but the server cannot be recovered or restarted.",
  },
  {
    state: "failed",
    color: "error",
    label: "Failed",
    description:
      "An unrecoverable error occurred during provisioning or operation. The server cannot self-heal from this state. Contact support at support@mahalaxmi.ai with your project name and the time the failure occurred.",
  },
];

const STATE_COLOR_MAP = {
  success: "success",
  warning: "warning",
  info: "info",
  error: "error",
  default: "default",
};

function SectionHeading({ children }) {
  return (
    <Typography variant="h5" component="h2" gutterBottom sx={{ mt: 5, mb: 1 }}>
      {children}
    </Typography>
  );
}

function SubHeading({ children }) {
  return (
    <Typography variant="h6" component="h3" gutterBottom sx={{ mt: 3, mb: 1 }}>
      {children}
    </Typography>
  );
}

function BodyText({ children, sx }) {
  return (
    <Typography variant="body1" color="text.secondary" sx={{ mb: 2, ...sx }}>
      {children}
    </Typography>
  );
}

export default function DocsCloudPage() {
  return (
    <Box
      component="article"
      sx={{
        maxWidth: 820,
        mx: "auto",
        px: { xs: 2, md: 4 },
        py: { xs: 4, md: 6 },
      }}
    >
      <Typography variant="overline" color="primary" display="block" gutterBottom>
        Docs / Cloud
      </Typography>

      <Typography variant="h3" component="h1" gutterBottom>
        Cloud
      </Typography>

      <BodyText>
        Mahalaxmi Cloud gives you a fully managed orchestration environment on dedicated Hetzner
        infrastructure. This page covers everything you need to set up a cloud server, understand
        its lifecycle, and manage it over time.
      </BodyText>

      <Divider sx={{ my: 4 }} />

      {/* ── 1. Setup ── */}
      <SectionHeading>1. Setup</SectionHeading>

      <BodyText>
        Getting a cloud server takes three steps: subscribe, pay, and wait for provisioning to
        complete.
      </BodyText>

      <SubHeading>Subscribe</SubHeading>
      <BodyText>
        Navigate to <strong>/cloud/pricing</strong> and choose a plan. Each plan maps to a specific
        Hetzner instance type. Click <em>Subscribe</em> to open the Stripe hosted checkout.
      </BodyText>

      <SubHeading>Payment</SubHeading>
      <BodyText>
        Stripe processes your payment. Mahalaxmi never stores card details — all billing data is
        held exclusively by Stripe. Once the payment is confirmed, the checkout redirects you back
        to the dashboard and provisioning begins automatically.
      </BodyText>

      <SubHeading>Provisioning</SubHeading>
      <BodyText>
        After payment is confirmed, Hetzner creates your virtual machine. The server status shows{" "}
        <Chip label="provisioning" size="small" color="info" /> during this phase. When the VM is
        ready and all health checks pass, the status transitions to{" "}
        <Chip label="active" size="small" color="success" />. This typically takes 1–3 minutes.
      </BodyText>

      <Divider sx={{ my: 4 }} />

      {/* ── 2. Project Name ── */}
      <SectionHeading>2. Project Name</SectionHeading>

      <BodyText>
        Your project name is chosen at subscription time and cannot be changed afterwards.
      </BodyText>

      <Box
        component="ul"
        sx={{ pl: 3, mb: 2, "& li": { mb: 1 } }}
      >
        <Typography component="li" variant="body1" color="text.secondary">
          <strong>Character set:</strong> lowercase letters <code>a–z</code>, digits{" "}
          <code>0–9</code>, and hyphens <code>-</code>.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          <strong>Length:</strong> 3 to 40 characters.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          <strong>Hyphens:</strong> must not appear at the start or end of the name.
        </Typography>
      </Box>

      <BodyText>
        The project name becomes the subdomain prefix of your fully qualified domain name (FQDN).
        For example, a project named <code>my-project</code> will be accessible at{" "}
        <code>my-project.mahalaxmi.ai</code>. Choose a name that is descriptive, concise, and
        unique to your organisation.
      </BodyText>

      <Alert severity="info" sx={{ mb: 2 }}>
        Because the project name becomes a public DNS label, avoid including personally identifiable
        information or internal system names in it.
      </Alert>

      <Divider sx={{ my: 4 }} />

      {/* ── 3. Lifecycle States ── */}
      <SectionHeading>3. Server Lifecycle States</SectionHeading>

      <BodyText>
        Every cloud server passes through a series of well-defined states. The dashboard always
        reflects the current state in real time.
      </BodyText>

      <TableContainer component={Paper} variant="outlined" sx={{ mb: 4 }}>
        <Table size="small" aria-label="Server lifecycle states">
          <TableHead>
            <TableRow>
              <TableCell>
                <strong>State</strong>
              </TableCell>
              <TableCell>
                <strong>Meaning</strong>
              </TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {LIFECYCLE_STATES.map(({ state, color, label, description }) => (
              <TableRow key={state} sx={{ verticalAlign: "top" }}>
                <TableCell sx={{ whiteSpace: "nowrap", py: 1.5 }}>
                  <Chip
                    label={label}
                    size="small"
                    color={STATE_COLOR_MAP[color]}
                    sx={{ fontFamily: "monospace" }}
                  />
                </TableCell>
                <TableCell sx={{ py: 1.5 }}>
                  <Typography variant="body2" color="text.secondary">
                    {description}
                  </Typography>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>

      <SubHeading>Stopped vs Failed</SubHeading>
      <BodyText>
        The distinction between <Chip label="stopped" size="small" /> and{" "}
        <Chip label="failed" size="small" color="error" /> is important:
      </BodyText>
      <Box
        component="ul"
        sx={{ pl: 3, mb: 2, "& li": { mb: 1 } }}
      >
        <Typography component="li" variant="body1" color="text.secondary">
          <strong>Stopped</strong> is a recoverable state triggered intentionally (manually or by
          idle timeout). The VM is gone but the subscription and configuration are intact. You can
          restart the server at any time.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          <strong>Failed</strong> is an unrecoverable state caused by an unexpected infrastructure
          or provisioning error. The server cannot self-heal. Contact{" "}
          <strong>support@mahalaxmi.ai</strong> for assistance.
        </Typography>
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* ── 4. Idle Timeout ── */}
      <SectionHeading>4. Idle Timeout</SectionHeading>

      <BodyText>
        To reduce unnecessary compute costs, Mahalaxmi monitors heartbeat activity on active
        servers. When no meaningful activity is detected for a configured quiet period, the server
        is automatically stopped.
      </BodyText>

      <BodyText>
        The auto-stop follows the same path as a manual stop: the server transitions through{" "}
        <Chip label="stopping" size="small" /> and lands in{" "}
        <Chip label="stopped" size="small" />. Your subscription is preserved and you can restart
        the server immediately from the dashboard.
      </BodyText>

      <Alert severity="warning" sx={{ mb: 2 }}>
        Any in-flight tasks running on the server when the idle timeout triggers will be
        interrupted. Ensure long-running workers emit regular heartbeat signals to prevent
        unexpected auto-stops.
      </Alert>

      <Divider sx={{ my: 4 }} />

      {/* ── 5. Restart ── */}
      <SectionHeading>5. Restarting a Stopped Server</SectionHeading>

      <BodyText>
        A server in the <Chip label="stopped" size="small" /> state can be restarted at any time.
      </BodyText>

      <Box
        component="ol"
        sx={{ pl: 3, mb: 2, "& li": { mb: 1 } }}
      >
        <Typography component="li" variant="body1" color="text.secondary">
          Open the dashboard and locate your project card.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          Click <strong>Restart</strong>. The server status will immediately change to{" "}
          <Chip label="provisioning" size="small" color="info" />.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          Wait 1–3 minutes for the VM to be created and health checks to pass. The status will
          transition to <Chip label="active" size="small" color="success" /> when ready.
        </Typography>
      </Box>

      <BodyText>
        Restarting a stopped server does not change your billing cycle — the subscription continues
        uninterrupted.
      </BodyText>

      <Divider sx={{ my: 4 }} />

      {/* ── 6. Delete ── */}
      <SectionHeading>6. Deleting a Server</SectionHeading>

      <Alert severity="error" sx={{ mb: 3 }}>
        <strong>Deletion is permanent and non-recoverable.</strong> All data stored on the server
        will be destroyed. This action cannot be undone.
      </Alert>

      <BodyText>
        To delete a server, open the project card in the dashboard and click <strong>Delete</strong>
        . You will be asked to confirm by typing your project name. Once confirmed:
      </BodyText>

      <Box
        component="ol"
        sx={{ pl: 3, mb: 2, "& li": { mb: 1 } }}
      >
        <Typography component="li" variant="body1" color="text.secondary">
          The server status changes to <Chip label="deleting" size="small" color="error" />.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          All Hetzner resources — VM, volumes, and network interfaces — are destroyed
          asynchronously.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          When complete, the status changes to <Chip label="deleted" size="small" color="error" />.
          The project entry is kept for billing history but no further operations are possible.
        </Typography>
      </Box>

      <BodyText>
        Deleting a server does <strong>not</strong> automatically cancel your Stripe subscription.
        To avoid future charges, cancel your subscription separately via the billing portal
        accessible from the dashboard.
      </BodyText>
    </Box>
  );
}
