// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightContextualMenu from 'starlight-contextual-menu';
import starlightChangelogs from 'starlight-changelogs';
import starlightLinksValidator from 'starlight-links-validator';
import starlightLlmsTxt from 'starlight-llms-txt';
import starlightSiteGraph from 'starlight-site-graph';
import starWarp from '@inox-tools/star-warp';

// https://astro.build/config
export default defineConfig({
	site: 'https://conclaude.dev',
	integrations: [
		starlight({
			title: 'conclaude',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/connerohnesorge/conclaude' }],
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Example Guide', slug: 'guides/example' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Example', slug: 'reference/example' },
						{
							label: 'Configuration Reference',
							items: [
								{ label: 'Configuration Overview', slug: 'reference/config/configuration' },
								{ label: 'Stop Hook', slug: 'reference/config/stop' },
								{ label: 'Subagent Stop Hook', slug: 'reference/config/subagent-stop' },
								{ label: 'Pre Tool Use Hook', slug: 'reference/config/pre-tool-use' },
								{ label: 'Notifications', slug: 'reference/config/notifications' },
								{ label: 'Permission Request Hook', slug: 'reference/config/permission-request' },
							],
						},
					],
				},
				{
					label: 'Changelog',
					link: '/changelog/',
				},
			],
			plugins: [
				starlightContextualMenu({
					actions: ['copy', 'view', 'chatgpt', 'claude'],
				}),
				starlightChangelogs(),
				starlightLinksValidator(),
				starlightLlmsTxt(),
				starlightSiteGraph(),
				starWarp(),
			],
		}),
	],
});
