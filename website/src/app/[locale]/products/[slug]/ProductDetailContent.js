'use client';

import React, { useState } from 'react';
import Image from 'next/image';
import {
  Container,
  Typography,
  Box,
  Button,
  Grid,
  Card,
  CardContent,
  Chip,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Alert,
  Breadcrumbs,
  Link as MuiLink,
  alpha,
  useTheme,
  Fade,
  Grow,
  Divider,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
} from '@mui/material';
import {
  ExpandMore,
  ArrowBack,
  CheckCircle,
  Rocket,
  AutoAwesome,
  Speed,
  Security,
  ArrowForward,
  Star,
  TrendingUp,
  School,
  Business,
  Person,
  AllInclusive,
  Download,
  OpenInNew,
} from '@mui/icons-material';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { releasesAPI } from '@/lib/api';
import CloudProviderTabs from './CloudProviderTabs';
import ComingSoonTab from './ComingSoonTab';
import BuyNowButton from '../../cloud/pricing/BuyNowButton';

const ProductDetailContent = ({ product, slug, providerLabels = {} }) => {
  const router = useRouter();
  const theme = useTheme();

  // Check if releases are available for download
  const { data: releaseData } = useQuery({
    queryKey: ['releases', 'latest'],
    queryFn: () => releasesAPI.getLatest(),
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
  const releasesAvailable = releaseData?.data?.success === true || !!product.always_downloadable;

  const isHeadlessProduct = slug === 'mahalaxmi-headless-orchestration';
  const [cloudProvider, setCloudProvider] = useState('hetzner');

  const pricingOptions = product.pricing_options || [];

  // Determine if pricing should be shown based on product and platform status
  const isPlatformConnected = product.is_platform_connected !== false;
  const isProductActive = product.is_active !== false;
  const hasPricingOptions = pricingOptions.length > 0;
  const platformStatusMessage = product.platform_status_message;

  // Show pricing only if: product is active AND (has pricing OR is intentionally contact-only)
  const showPricing = isProductActive && hasPricingOptions;
  // Show contact support message if: product not active OR platform down with no pricing
  const showContactSupport = !isProductActive || (!hasPricingOptions && platformStatusMessage);

  const getPlanIcon = (planName) => {
    const name = planName.toLowerCase();
    if (name.includes('enterprise')) return <Business sx={{ fontSize: 32 }} />;
    if (name.includes('lifetime')) return <AllInclusive sx={{ fontSize: 32 }} />;
    if (name.includes('student')) return <School sx={{ fontSize: 32 }} />;
    return <Person sx={{ fontSize: 32 }} />;
  };

  const formatPrice = (option) => {
    if (option.price_period === 'one-time') {
      return { price: option.price, period: '' };
    }
    if (option.price_period === 'seat/year') {
      return { price: option.price, period: '/seat/yr' };
    }
    if (option.price_period === 'year') {
      return { price: option.price, period: '/yr' };
    }
    return { price: option.price, period: '/mo' };
  };

  // Get trial button configuration based on product trial_type
  const getTrialButtonProps = () => {
    const trialType = product.trial_type || 'contact';
    const buttonText = product.trial_button_text || 'Start Free Trial';

    if (trialType === 'download') {
      // Always use releasesAPI.getDownloadUrl() which constructs the full URL with API_BASE_URL
      // This ensures the download link points to the correct backend server
      if (releasesAvailable || product.release_available) {
        return {
          component: 'a',
          href: releasesAPI.getDownloadUrl(),
          download: true,
          text: buttonText,
          icon: <Download />,
        };
      }
      // No releases available - fall back to contact
      return {
        component: Link,
        href: '/contact',
        text: 'Contact for Trial',
        icon: <ArrowForward />,
      };
    }

    if (trialType === 'external_redirect' && product.trial_redirect_url) {
      return {
        component: 'a',
        href: product.trial_redirect_url,
        target: '_blank',
        rel: 'noopener noreferrer',
        text: buttonText,
        icon: <OpenInNew />,
      };
    }

    // Default: contact page
    return {
      component: Link,
      href: '/contact',
      text: buttonText,
      icon: <ArrowForward />,
    };
  };

  const trialButtonProps = getTrialButtonProps();

  return (
    <>
      {/* Hero Section */}
      <Box
        sx={{
          background: `linear-gradient(135deg, ${alpha(theme.palette.primary.dark, 0.95)} 0%, ${alpha('#1a1a2e', 0.98)} 100%)`,
          color: 'white',
          pt: { xs: 4, md: 6 },
          pb: { xs: 8, md: 12 },
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        <Container maxWidth="lg" sx={{ position: 'relative', zIndex: 1 }}>
          {/* Breadcrumbs */}
          <Fade in timeout={500}>
            <Breadcrumbs
              sx={{
                mb: 4,
                '& .MuiBreadcrumbs-separator': { color: 'rgba(255,255,255,0.5)' },
              }}
            >
              <MuiLink
                component={Link}
                href="/"
                underline="hover"
                sx={{ color: 'rgba(255,255,255,0.7)', '&:hover': { color: 'white' } }}
              >
                Home
              </MuiLink>
              <MuiLink
                component={Link}
                href="/products"
                underline="hover"
                sx={{ color: 'rgba(255,255,255,0.7)', '&:hover': { color: 'white' } }}
              >
                Products
              </MuiLink>
              <Typography sx={{ color: 'white' }}>{product.name}</Typography>
            </Breadcrumbs>
          </Fade>

          <Grid container spacing={6} alignItems="center">
            <Grid item xs={12} md={7}>
              <Fade in timeout={800}>
                <Box>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 3, flexWrap: 'wrap' }}>
                    {product.is_featured && (
                      <Chip
                        icon={<AutoAwesome sx={{ fontSize: 18 }} />}
                        label="Featured Product"
                        sx={{
                          background: 'linear-gradient(90deg, #ffd700 0%, #ffaa00 100%)',
                          color: '#000',
                          fontWeight: 600,
                          px: 1,
                        }}
                      />
                    )}
                  </Box>

                  <Typography
                    variant="h1"
                    component="h1"
                    sx={{
                      fontWeight: 800,
                      fontSize: { xs: '2.5rem', md: '3.5rem' },
                      mb: 2,
                      lineHeight: 1.2,
                    }}
                  >
                    {product.name}
                  </Typography>

                  {product.tagline && (
                    <Typography
                      variant="h5"
                      sx={{
                        color: '#ffd700',
                        fontWeight: 500,
                        mb: 2,
                      }}
                    >
                      {product.tagline}
                    </Typography>
                  )}

                  <Typography
                    variant="h6"
                    sx={{
                      opacity: 0.9,
                      fontWeight: 300,
                      lineHeight: 1.6,
                      mb: 4,
                    }}
                  >
                    {product.short_description}
                  </Typography>

                  <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
                    {showPricing ? (
                      <>
                        <Button
                          variant="contained"
                          size="large"
                          href="#pricing"
                          endIcon={<ArrowForward />}
                          sx={{
                            bgcolor: 'white',
                            color: 'primary.main',
                            px: 4,
                            py: 1.5,
                            fontWeight: 600,
                            '&:hover': {
                              bgcolor: 'rgba(255,255,255,0.08)',
                            },
                          }}
                        >
                          View Pricing
                        </Button>
                        <Button
                          variant="outlined"
                          size="large"
                          component={Link}
                          href="/contact"
                          sx={{
                            borderColor: 'rgba(255,255,255,0.5)',
                            color: 'white',
                            px: 4,
                            py: 1.5,
                            '&:hover': {
                              borderColor: 'white',
                              bgcolor: 'rgba(255,255,255,0.1)',
                            },
                          }}
                        >
                          Contact Sales
                        </Button>
                      </>
                    ) : (
                      <Button
                        variant="contained"
                        size="large"
                        component={Link}
                        href="/contact"
                        endIcon={<ArrowForward />}
                        sx={{
                          bgcolor: 'white',
                          color: 'primary.main',
                          px: 4,
                          py: 1.5,
                          fontWeight: 600,
                          '&:hover': {
                            bgcolor: 'rgba(255,255,255,0.08)',
                          },
                        }}
                      >
                        Contact Support
                      </Button>
                    )}
                  </Box>
                </Box>
              </Fade>
            </Grid>

            <Grid item xs={12} md={5}>
              <Grow in timeout={1000}>
                <Box
                  sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'center',
                    position: 'relative',
                    minHeight: 300,
                  }}
                >
                  {product.image ? (
                    <Image
                      src={product.image}
                      alt={product.name}
                      fill
                      sizes="(max-width: 600px) 90vw, 400px"
                      style={{ objectFit: 'contain' }}
                      unoptimized
                    />
                  ) : (
                    <Box
                      sx={{
                        width: 200,
                        height: 200,
                        borderRadius: 4,
                        background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.primary.dark} 100%)`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        boxShadow: `0 30px 60px ${alpha('#000', 0.4)}`,
                      }}
                    >
                      <Rocket sx={{ fontSize: 100, color: 'white' }} />
                    </Box>
                  )}
                </Box>
              </Grow>
            </Grid>
          </Grid>
        </Container>
      </Box>

      {/* About Section - Long Description */}
      {product.long_description && (
        <Container maxWidth="lg" sx={{ py: { xs: 6, md: 10 } }}>
          <Grid container spacing={6}>
            <Grid item xs={12} md={8}>
              <Typography variant="h3" sx={{ fontWeight: 700, mb: 4 }}>
                About {product.name}
              </Typography>
              <Box
                sx={{
                  '& p': { mb: 2, lineHeight: 1.8, color: 'text.secondary' },
                  '& strong': { color: 'text.primary', fontWeight: 600 },
                  '& ul, & li': { color: 'text.secondary', lineHeight: 1.8 },
                }}
              >
                {product.long_description.split('\n\n').map((paragraph, idx) => {
                  if (paragraph.startsWith('**') && paragraph.endsWith('**')) {
                    return (
                      <Typography key={idx} variant="h5" sx={{ fontWeight: 600, mt: 4, mb: 2, color: 'text.primary' }}>
                        {paragraph.replace(/\*\*/g, '')}
                      </Typography>
                    );
                  }
                  if (paragraph.startsWith('- ')) {
                    const items = paragraph.split('\n').filter(line => line.startsWith('- '));
                    return (
                      <Box key={idx} component="ul" sx={{ pl: 2, mb: 2 }}>
                        {items.map((item, itemIdx) => (
                          <Typography key={itemIdx} component="li" variant="body1" sx={{ mb: 1 }}>
                            {item.replace('- ', '')}
                          </Typography>
                        ))}
                      </Box>
                    );
                  }
                  return (
                    <Typography key={idx} variant="body1" sx={{ mb: 2, lineHeight: 1.8 }}>
                      {paragraph.split('**').map((part, partIdx) =>
                        partIdx % 2 === 1 ? (
                          <strong key={partIdx}>{part}</strong>
                        ) : (
                          part
                        )
                      )}
                    </Typography>
                  );
                })}
              </Box>
            </Grid>
            <Grid item xs={12} md={4}>
              <Card sx={{ borderRadius: 3, p: 3, bgcolor: alpha(theme.palette.primary.main, 0.03), border: '1px solid', borderColor: 'divider' }}>
                <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>
                  Quick Facts
                </Typography>
                {product.specifications && Object.entries(product.specifications).slice(0, 5).map(([key, value]) => (
                  <Box key={key} sx={{ display: 'flex', justifyContent: 'space-between', py: 1.5, borderBottom: '1px solid', borderColor: 'divider' }}>
                    <Typography variant="body2" color="text.secondary">{key}</Typography>
                    <Typography variant="body2" sx={{ fontWeight: 600 }}>{value}</Typography>
                  </Box>
                ))}
                {showPricing ? (
                  <Button
                    href="#pricing"
                    variant="contained"
                    fullWidth
                    size="large"
                    sx={{ mt: 3, fontWeight: 600 }}
                  >
                    View Pricing
                  </Button>
                ) : (
                  <Button
                    component={Link}
                    href="/contact"
                    variant="contained"
                    fullWidth
                    size="large"
                    sx={{ mt: 3, fontWeight: 600 }}
                  >
                    Contact Support
                  </Button>
                )}
              </Card>
            </Grid>
          </Grid>
        </Container>
      )}

      {/* Pricing Section - Only show if product is active and has pricing options */}
      {showPricing ? (
        <Box id="pricing" sx={{ py: { xs: 8, md: 12 }, bgcolor: 'rgba(255,255,255,0.05)' }}>
          <Container maxWidth="lg">

            {isHeadlessProduct && (
              <CloudProviderTabs onProviderChange={setCloudProvider} providerLabels={providerLabels} />
            )}

            {!isHeadlessProduct || cloudProvider === 'hetzner' ? (
              <>
            <Grid container spacing={3} justifyContent="center">
              {pricingOptions.map((option, index) => {
                const { price, period } = formatPrice(option);
                const isPopular = option.is_popular;

                return (
                  <Grid item xs={12} sm={6} md={4} lg={true} key={option.id}>
                    <Grow in timeout={600 + index * 100}>
                      <Card
                        sx={{
                          height: '100%',
                          borderRadius: 4,
                          border: '2px solid',
                          borderColor: isPopular ? 'primary.main' : 'divider',
                          position: 'relative',
                          overflow: 'visible',
                          transition: 'all 0.3s ease',
                          transform: isPopular ? 'scale(1.02)' : 'none',
                          '&:hover': {
                            transform: isPopular ? 'scale(1.04)' : 'translateY(-8px)',
                            boxShadow: `0 20px 40px ${alpha(theme.palette.primary.main, 0.2)}`,
                          },
                        }}
                      >
                        {isPopular && (
                          <Chip
                            label="Most Popular"
                            color="primary"
                            sx={{
                              position: 'absolute',
                              top: -12,
                              left: '50%',
                              transform: 'translateX(-50%)',
                              fontWeight: 600,
                            }}
                          />
                        )}

                        <CardContent sx={{ p: 4 }}>
                          <Box sx={{ textAlign: 'center', mb: 3 }}>
                            <Box
                              sx={{
                                width: 60,
                                height: 60,
                                borderRadius: '50%',
                                bgcolor: alpha(theme.palette.primary.main, 0.1),
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                mx: 'auto',
                                mb: 2,
                                color: 'primary.main',
                              }}
                            >
                              {getPlanIcon(option.name)}
                            </Box>
                            <Typography variant="h5" sx={{ fontWeight: 700 }}>
                              {option.name}
                            </Typography>
                            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                              {option.description}
                            </Typography>
                          </Box>

                          <Box sx={{ textAlign: 'center', mb: 3 }}>
                            <Typography
                              variant="h3"
                              sx={{ fontWeight: 800, color: 'primary.main' }}
                            >
                              ${price}
                              <Typography
                                component="span"
                                variant="body1"
                                color="text.secondary"
                              >
                                {period}
                              </Typography>
                            </Typography>
                            {option.price_monthly_equivalent && (
                              <Typography variant="body2" color="success.main" sx={{ fontWeight: 600 }}>
                                ~${option.price_monthly_equivalent}/mo
                              </Typography>
                            )}
                            {option.savings && (
                              <Chip
                                label={option.savings}
                                size="small"
                                color="success"
                                sx={{ mt: 1 }}
                              />
                            )}
                            {option.minimum_seats && (
                              <Typography variant="caption" display="block" color="text.secondary" sx={{ mt: 1 }}>
                                {option.minimum_seats} seat minimum
                              </Typography>
                            )}
                          </Box>

                          <Divider sx={{ my: 3 }} />

                          <Box sx={{ mb: 3 }}>
                            {option.features?.map((feature, idx) => (
                              <Box
                                key={idx}
                                sx={{
                                  display: 'flex',
                                  alignItems: 'flex-start',
                                  gap: 1,
                                  mb: 1.5,
                                }}
                              >
                                <CheckCircle
                                  sx={{ fontSize: 18, color: 'success.main', mt: 0.3 }}
                                />
                                <Typography variant="body2">{typeof feature === 'object' ? feature.name : feature}</Typography>
                              </Box>
                            ))}
                          </Box>

                          {/* Eligibility Requirements */}
                          {option.eligibility && option.eligibility.length > 0 && (
                            <Box sx={{ mb: 3, p: 2, bgcolor: alpha(theme.palette.info.main, 0.08), borderRadius: 2 }}>
                              <Typography variant="caption" color="info.main" sx={{ fontWeight: 600, display: 'block', mb: 1 }}>
                                Eligible for:
                              </Typography>
                              {option.eligibility.map((item, idx) => (
                                <Typography key={idx} variant="body2" color="text.secondary" sx={{ fontSize: '0.8rem' }}>
                                  • {typeof item === 'object' ? item.name : item}
                                </Typography>
                              ))}
                            </Box>
                          )}

                          {(() => {
                            // Fallback: free tiers (price 0) default to download if API omits cta_action
                            const ctaAction = option.cta_action ?? option.ctaAction ?? (option.price === 0 ? 'download' : 'checkout');
                            const ctaLabel  = option.cta_label  ?? option.ctaLabel ?? option.name ?? 'Get started';
                            const variant   = isPopular ? 'contained' : 'outlined';
                            if (ctaAction === 'download') {
                              const downloadHref = releasesAvailable ? '/download' : '/contact';
                              return (
                                <Button component={Link} href={downloadHref} variant={variant} fullWidth sx={{ fontWeight: 600 }}>
                                  {ctaLabel}
                                </Button>
                              );
                            }
                            if (ctaAction === 'contact') {
                              return (
                                <Button component={Link} href="/contact" variant={variant} fullWidth sx={{ fontWeight: 600 }}>
                                  {ctaLabel}
                                </Button>
                              );
                            }
                            if (ctaAction === 'verify') {
                              return (
                                <Button component={Link} href="/contact?subject=student-license" variant={variant} fullWidth sx={{ fontWeight: 600 }}>
                                  {ctaLabel}
                                </Button>
                              );
                            }
                            if (ctaAction === 'addon' || option.isAddon) {
                              return (
                                <button
                                  onClick={() => window.open('https://mahalaxmi.ai/dashboard', '_blank')}
                                  style={{ width: '100%', padding: '12px', fontWeight: 600, borderRadius: 8, cursor: 'pointer', border: '1px solid', borderColor: 'rgba(0,0,0,0.23)', background: 'transparent', fontSize: '0.9375rem' }}
                                >
                                  Add to my plan
                                </button>
                              );
                            }
                            // checkout (or unknown) — use BuyNowButton
                            return (
                              <BuyNowButton
                                tier={option.slug ?? option.id}
                                label={ctaLabel}
                                variant={variant}
                              />
                            );
                          })()}
                        </CardContent>
                      </Card>
                    </Grow>
                  </Grid>
                );
              })}
            </Grid>

            {/* Volume Discounts for Enterprise */}
            {pricingOptions.find(p => p.volume_discounts?.length > 0) && (
              <Box sx={{ mt: 8 }}>
                <Typography variant="h4" sx={{ fontWeight: 700, mb: 4, textAlign: 'center' }}>
                  Enterprise Volume Discounts
                </Typography>
                <TableContainer component={Paper} sx={{ maxWidth: 600, mx: 'auto', borderRadius: 3 }}>
                  <Table>
                    <TableHead>
                      <TableRow sx={{ bgcolor: 'primary.main' }}>
                        <TableCell sx={{ color: 'white', fontWeight: 600 }}>Seats</TableCell>
                        <TableCell sx={{ color: 'white', fontWeight: 600 }}>Discount</TableCell>
                        <TableCell sx={{ color: 'white', fontWeight: 600 }}>Price/Seat</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {pricingOptions
                        .find(p => p.volume_discounts?.length > 0)
                        ?.volume_discounts.map((tier, idx) => (
                          <TableRow key={idx} sx={{ '&:hover': { bgcolor: 'rgba(255,255,255,0.05)' } }}>
                            <TableCell sx={{ fontWeight: 500 }}>
                              {tier.maxSeats ? `${tier.minSeats}-${tier.maxSeats}` : `${tier.minSeats}+`}
                            </TableCell>
                            <TableCell>
                              {tier.discountPercent > 0 ? (
                                <Chip label={`${tier.discountPercent}%`} size="small" color="success" />
                              ) : (
                                '—'
                              )}
                            </TableCell>
                            <TableCell sx={{ fontWeight: 700, color: 'primary.main' }}>
                              ${tier.pricePerSeat}/seat/yr
                            </TableCell>
                          </TableRow>
                        ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              </Box>
            )}
              </>
            ) : (
              <ComingSoonTab provider={cloudProvider} />
            )}
          </Container>
        </Box>
      ) : showContactSupport ? (
        /* Contact Support Section - Product inactive or platform unavailable */
        <Box id="pricing" sx={{ py: { xs: 8, md: 12 }, bgcolor: 'rgba(255,255,255,0.05)' }}>
          <Container maxWidth="md">
            <Box
              sx={{
                textAlign: 'center',
                p: { xs: 4, md: 6 },
                borderRadius: 4,
                bgcolor: 'white',
                border: '1px solid',
                borderColor: 'warning.light',
                boxShadow: `0 4px 20px rgba(237, 108, 2, 0.1)`,
              }}
            >
              <Box
                sx={{
                  width: 60,
                  height: 60,
                  borderRadius: '50%',
                  bgcolor: 'warning.light',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  mx: 'auto',
                  mb: 3,
                }}
              >
                <Security sx={{ fontSize: 32, color: 'warning.dark' }} />
              </Box>
              <Typography variant="h3" sx={{ fontWeight: 700, mb: 2 }}>
                Contact Support for Pricing
              </Typography>
              <Typography variant="h6" color="text.secondary" sx={{ fontWeight: 400, mb: 2, maxWidth: 600, mx: 'auto' }}>
                {platformStatusMessage || 'Pricing information is currently unavailable for this product.'}
              </Typography>
              <Typography variant="body1" color="text.secondary" sx={{ mb: 4, maxWidth: 500, mx: 'auto' }}>
                Our support team is ready to assist you with current pricing, availability, and any questions about {product.name}.
              </Typography>
              <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
                <Button
                  component={Link}
                  href="/contact"
                  variant="contained"
                  size="large"
                  endIcon={<ArrowForward />}
                  sx={{
                    px: 5,
                    py: 1.5,
                    fontWeight: 600,
                    borderRadius: 2,
                  }}
                >
                  Contact Support
                </Button>
                {product.support_email && (
                  <Button
                    component="a"
                    href={`mailto:${product.support_email}`}
                    variant="outlined"
                    size="large"
                    sx={{
                      px: 4,
                      py: 1.5,
                      fontWeight: 600,
                      borderRadius: 2,
                    }}
                  >
                    Email: {product.support_email}
                  </Button>
                )}
              </Box>
            </Box>
          </Container>
        </Box>
      ) : (
        /* Contact Us Section for services without pricing (intentionally contact-only) */
        <Box id="pricing" sx={{ py: { xs: 8, md: 12 }, bgcolor: 'rgba(255,255,255,0.05)' }}>
          <Container maxWidth="md">
            <Box
              sx={{
                textAlign: 'center',
                p: { xs: 4, md: 6 },
                borderRadius: 4,
                bgcolor: 'white',
                border: '1px solid',
                borderColor: 'divider',
              }}
            >
              <Typography variant="h2" sx={{ fontWeight: 700, mb: 2 }}>
                Let&apos;s Discuss Your Needs
              </Typography>
              <Typography variant="h6" color="text.secondary" sx={{ fontWeight: 400, mb: 4, maxWidth: 600, mx: 'auto' }}>
                Every project is unique. Contact us for a personalized consultation and custom quote tailored to your specific requirements.
              </Typography>
              <Button
                component={Link}
                href="/contact"
                variant="contained"
                size="large"
                endIcon={<ArrowForward />}
                sx={{
                  px: 5,
                  py: 1.5,
                  fontWeight: 600,
                  borderRadius: 2,
                }}
              >
                Contact Us
              </Button>
            </Box>
          </Container>
        </Box>
      )}

      {/* Features Section */}
      {product.features && product.features.length > 0 && (
        <Container maxWidth="lg" sx={{ py: { xs: 8, md: 12 } }}>
          <Typography variant="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            All Features Included
          </Typography>

          <Grid container spacing={3}>
            {product.features.map((feature, index) => (
              <Grid item xs={12} sm={6} md={4} key={index}>
                <Grow in timeout={500 + index * 100}>
                  <Box
                    sx={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: 2,
                      p: 3,
                      borderRadius: 3,
                      backgroundColor: alpha(theme.palette.success.main, 0.05),
                      border: '1px solid',
                      borderColor: alpha(theme.palette.success.main, 0.2),
                      transition: 'all 0.3s ease',
                      '&:hover': {
                        backgroundColor: alpha(theme.palette.success.main, 0.1),
                        transform: 'translateX(8px)',
                      },
                    }}
                  >
                    <CheckCircle sx={{ color: 'success.main', fontSize: 24, mt: 0.25 }} />
                    <Typography variant="body1" sx={{ fontWeight: 500 }}>
                      {typeof feature === 'object' ? feature.name : feature}
                    </Typography>
                  </Box>
                </Grow>
              </Grid>
            ))}
          </Grid>
        </Container>
      )}

      {/* Benefits Section */}
      {product.benefits && product.benefits.length > 0 && (
        <Box sx={{ bgcolor: 'rgba(255,255,255,0.05)', py: { xs: 8, md: 12 } }}>
          <Container maxWidth="lg">
            <Typography variant="h2" sx={{ fontWeight: 700, mb: 2, textAlign: 'center' }}>
              Why Choose {product.name}?
            </Typography>
            <Typography
              variant="h6"
              color="text.secondary"
              sx={{ mb: 6, textAlign: 'center', fontWeight: 400 }}
            >
              Built with cutting-edge technology to deliver exceptional results
            </Typography>

            <Grid container spacing={4}>
              {product.benefits.map((benefit, index) => (
                <Grid item xs={12} md={4} key={index}>
                  <Grow in timeout={600 + index * 150}>
                    <Card
                      sx={{
                        height: '100%',
                        textAlign: 'center',
                        p: 4,
                        borderRadius: 4,
                        border: '1px solid',
                        borderColor: 'divider',
                        transition: 'all 0.3s ease',
                        '&:hover': {
                          transform: 'translateY(-8px)',
                          boxShadow: `0 20px 40px ${alpha(theme.palette.primary.main, 0.15)}`,
                          borderColor: 'primary.main',
                        },
                      }}
                    >
                      <Box
                        sx={{
                          width: 70,
                          height: 70,
                          borderRadius: '50%',
                          background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.primary.main, 0.2)} 100%)`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          mx: 'auto',
                          mb: 3,
                          color: 'primary.main',
                        }}
                      >
                        {index === 0 && <Speed sx={{ fontSize: 32 }} />}
                        {index === 1 && <Security sx={{ fontSize: 32 }} />}
                        {index === 2 && <TrendingUp sx={{ fontSize: 32 }} />}
                      </Box>
                      <Typography variant="h5" sx={{ fontWeight: 600, mb: 2 }}>
                        {benefit.title}
                      </Typography>
                      <Typography variant="body1" color="text.secondary">
                        {benefit.description}
                      </Typography>
                    </Card>
                  </Grow>
                </Grid>
              ))}
            </Grid>
          </Container>
        </Box>
      )}

      {/* Specifications Section */}
      {product.specifications && Object.keys(product.specifications).length > 0 && (
        <Container maxWidth="lg" sx={{ py: { xs: 8, md: 12 } }}>
          <Typography variant="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            Technical Specifications
          </Typography>

          <Card sx={{ borderRadius: 4, overflow: 'hidden', maxWidth: 800, mx: 'auto' }}>
            {Object.entries(product.specifications).map(([key, value], index, arr) => (
              <Box key={key}>
                <Box
                  sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    p: 3,
                    '&:hover': { backgroundColor: alpha(theme.palette.primary.main, 0.03) },
                  }}
                >
                  <Typography variant="body1" color="text.secondary" sx={{ fontWeight: 500 }}>
                    {key}
                  </Typography>
                  <Typography variant="body1" sx={{ fontWeight: 600 }}>
                    {value}
                  </Typography>
                </Box>
                {index < arr.length - 1 && <Divider />}
              </Box>
            ))}
          </Card>
        </Container>
      )}

      {/* FAQs Section */}
      {product.faqs && product.faqs.length > 0 && (
        <Box sx={{ bgcolor: 'rgba(255,255,255,0.05)', py: { xs: 8, md: 12 } }}>
          <Container maxWidth="lg">
            <Typography variant="h2" sx={{ fontWeight: 700, mb: 2, textAlign: 'center' }}>
              Frequently Asked Questions
            </Typography>
            <Typography
              variant="h6"
              color="text.secondary"
              sx={{ mb: 6, textAlign: 'center', fontWeight: 400 }}
            >
              Got questions? We&apos;ve got answers
            </Typography>

            <Box sx={{ maxWidth: 800, mx: 'auto' }}>
              {product.faqs.map((faq, index) => (
                <Accordion
                  key={index}
                  sx={{
                    mb: 2,
                    borderRadius: '16px !important',
                    border: '1px solid',
                    borderColor: 'divider',
                    '&::before': { display: 'none' },
                    '&.Mui-expanded': {
                      boxShadow: `0 10px 40px ${alpha('#000', 0.1)}`,
                    },
                  }}
                >
                  <AccordionSummary
                    expandIcon={<ExpandMore />}
                    sx={{
                      px: 3,
                      '& .MuiAccordionSummary-content': { my: 2 },
                    }}
                  >
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <Box
                        sx={{
                          width: 32,
                          height: 32,
                          borderRadius: '50%',
                          backgroundColor: alpha(theme.palette.primary.main, 0.1),
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          mr: 2,
                        }}
                      >
                        <Star sx={{ fontSize: 18, color: 'primary.main' }} />
                      </Box>
                      <Typography sx={{ fontWeight: 600 }}>{faq.question}</Typography>
                    </Box>
                  </AccordionSummary>
                  <AccordionDetails sx={{ px: 3, pb: 3 }}>
                    <Typography variant="body1" color="text.secondary" sx={{ lineHeight: 1.8 }}>
                      {faq.answer}
                    </Typography>
                  </AccordionDetails>
                </Accordion>
              ))}
            </Box>
          </Container>
        </Box>
      )}

      {/* CTA Section */}
      <Container maxWidth="lg" sx={{ py: { xs: 8, md: 12 } }}>
        <Box
          sx={{
            p: { xs: 4, md: 8 },
            borderRadius: 4,
            background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.primary.dark} 100%)`,
            color: 'white',
            textAlign: 'center',
          }}
        >
          <TrendingUp sx={{ fontSize: 60, mb: 2, opacity: 0.9 }} />
          <Typography variant="h3" sx={{ fontWeight: 700, mb: 2 }}>
            {showContactSupport ? 'Have Questions?' : 'Ready to Get Started?'}
          </Typography>
          <Typography variant="h6" sx={{ opacity: 0.9, mb: 4, fontWeight: 300, maxWidth: 600, mx: 'auto' }}>
            {showContactSupport
              ? 'Our team is here to help you find the right solution for your needs.'
              : 'Start your 30-day free trial today. No credit card required.'}
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
            {showContactSupport ? (
              <Button
                component={Link}
                href="/contact"
                variant="contained"
                size="large"
                endIcon={<ArrowForward />}
                sx={{
                  bgcolor: 'white',
                  color: 'primary.main',
                  px: 5,
                  py: 1.5,
                  fontWeight: 600,
                  '&:hover': { bgcolor: 'rgba(255,255,255,0.08)' },
                }}
              >
                Contact Support
              </Button>
            ) : (
              <Button
                component={trialButtonProps.component}
                {...(trialButtonProps.href ? { href: trialButtonProps.href } : {})}
                {...(trialButtonProps.target ? { target: trialButtonProps.target, rel: trialButtonProps.rel } : {})}
                {...(trialButtonProps.download ? { download: true } : {})}
                variant="contained"
                size="large"
                endIcon={trialButtonProps.icon}
                sx={{
                  bgcolor: 'white',
                  color: 'primary.main',
                  px: 5,
                  py: 1.5,
                  fontWeight: 600,
                  '&:hover': { bgcolor: 'rgba(255,255,255,0.08)' },
                }}
              >
                {trialButtonProps.text}
              </Button>
            )}
            <Button
              onClick={() => router.push('/products')}
              variant="outlined"
              size="large"
              startIcon={<ArrowBack />}
              sx={{
                borderColor: 'rgba(255,255,255,0.5)',
                color: 'white',
                px: 4,
                py: 1.5,
                '&:hover': {
                  borderColor: 'white',
                  bgcolor: 'rgba(255,255,255,0.1)',
                },
              }}
            >
              View All Products
            </Button>
          </Box>
        </Box>
      </Container>
    </>
  );
};

export default ProductDetailContent;
