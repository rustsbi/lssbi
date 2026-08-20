// SPDX-License-Identifier: GPL-2.0-only
#include <linux/errno.h>
#include <linux/init.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/smp.h>
#include <asm/sbi.h>

#define LSSBI_EXT_TIME		0x54494D45UL
#define LSSBI_EXT_IPI		0x735049UL
#define LSSBI_EXT_RFENCE	0x52464E43UL
#define LSSBI_EXT_HSM		0x48534DUL
#define LSSBI_EXT_SRST		0x53525354UL
#define LSSBI_EXT_PMU		0x504D55UL
#define LSSBI_EXT_DBCN		0x4442434EUL
#define LSSBI_EXT_SUSP		0x53555350UL
#define LSSBI_EXT_CPPC		0x43505043UL
#define LSSBI_EXT_NACL		0x4E41434CUL
#define LSSBI_EXT_STA		0x535441UL
#define LSSBI_EXT_SSE		0x535345UL
#define LSSBI_EXT_FWFT		0x46574654UL
#define LSSBI_EXT_DBTR		0x44425452UL
#define LSSBI_EXT_MPXY		0x4D505859UL
#define LSSBI_EXT_FWFT_GET	1UL

#define LSSBI_EXTENSIONS_BUFFER_SIZE	2048
#define LSSBI_FWFT_BUFFER_SIZE		1024

static unsigned long spec_version;
static unsigned long impl_id;
static unsigned long impl_version;
static unsigned long mvendorid;
static unsigned long marchid;
static unsigned long mimpid;

module_param(spec_version, ulong, 0444);
MODULE_PARM_DESC(spec_version, "Raw SBI specification version");
module_param(impl_id, ulong, 0444);
MODULE_PARM_DESC(impl_id, "SBI implementation ID");
module_param(impl_version, ulong, 0444);
MODULE_PARM_DESC(impl_version, "Raw SBI implementation version");
module_param(mvendorid, ulong, 0444);
MODULE_PARM_DESC(mvendorid, "Raw RISC-V machine vendor ID");
module_param(marchid, ulong, 0444);
MODULE_PARM_DESC(marchid, "Raw RISC-V machine architecture ID");
module_param(mimpid, ulong, 0444);
MODULE_PARM_DESC(mimpid, "Raw RISC-V machine implementation ID");

struct lssbi_sbiret {
	long error;
	long value;
};

struct lssbi_probe_item {
	const char *name;
	unsigned long id;
};

static const struct lssbi_probe_item lssbi_extensions[] = {
	{ "base", SBI_EXT_BASE },
	{ "time", LSSBI_EXT_TIME },
	{ "ipi", LSSBI_EXT_IPI },
	{ "rfence", LSSBI_EXT_RFENCE },
	{ "hsm", LSSBI_EXT_HSM },
	{ "srst", LSSBI_EXT_SRST },
	{ "pmu", LSSBI_EXT_PMU },
	{ "dbcn", LSSBI_EXT_DBCN },
	{ "susp", LSSBI_EXT_SUSP },
	{ "cppc", LSSBI_EXT_CPPC },
	{ "nacl", LSSBI_EXT_NACL },
	{ "sta", LSSBI_EXT_STA },
	{ "sse", LSSBI_EXT_SSE },
	{ "fwft", LSSBI_EXT_FWFT },
	{ "dbtr", LSSBI_EXT_DBTR },
	{ "mpxy", LSSBI_EXT_MPXY },
	{ "legacy_set_timer", 0 },
	{ "legacy_console_putchar", 1 },
	{ "legacy_console_getchar", 2 },
	{ "legacy_clear_ipi", 3 },
	{ "legacy_send_ipi", 4 },
	{ "legacy_remote_fence_i", 5 },
	{ "legacy_remote_sfence_vma", 6 },
	{ "legacy_remote_sfence_vma_asid", 7 },
	{ "legacy_shutdown", 8 },
};

static const struct lssbi_probe_item lssbi_fwft_features[] = {
	{ "misaligned_exc_deleg", 0 },
	{ "landing_pad", 1 },
	{ "shadow_stack", 2 },
	{ "double_trap", 3 },
	{ "pte_ad_hw_updating", 4 },
	{ "pointer_masking_pmlen", 5 },
};

static struct lssbi_sbiret
lssbi_extension_results[ARRAY_SIZE(lssbi_extensions)];

/*
 * Keep the raw call private because __sbi_ecall() is not exported to modules.
 * The generic EID/FID form supports both cached Base/extension queries and
 * live extension-specific reads such as FWFT GET.
 */
static struct lssbi_sbiret lssbi_ecall(unsigned long extension_id,
				       unsigned long function_id,
				       unsigned long arg0,
				       unsigned long arg1,
				       unsigned long arg2,
				       unsigned long arg3,
				       unsigned long arg4,
				       unsigned long arg5)
{
	register unsigned long a0 asm("a0") = arg0;
	register unsigned long a1 asm("a1") = arg1;
	register unsigned long a2 asm("a2") = arg2;
	register unsigned long a3 asm("a3") = arg3;
	register unsigned long a4 asm("a4") = arg4;
	register unsigned long a5 asm("a5") = arg5;
	register unsigned long a6 asm("a6") = function_id;
	register unsigned long a7 asm("a7") = extension_id;

	asm volatile("ecall"
		     : "+r" (a0), "+r" (a1)
		     : "r" (a2), "r" (a3), "r" (a4), "r" (a5),
		       "r" (a6), "r" (a7)
		     : "memory");

	return (struct lssbi_sbiret) {
		.error = (long)a0,
		.value = (long)a1,
	};
}

static int lssbi_read_base(unsigned long function_id, unsigned long *value)
{
	struct lssbi_sbiret ret;

	ret = lssbi_ecall(SBI_EXT_BASE, function_id, 0, 0, 0, 0, 0, 0);
	if (ret.error)
		return -EOPNOTSUPP;

	*value = (unsigned long)ret.value;
	return 0;
}

static int lssbi_param_get_extensions(char *buffer,
				      const struct kernel_param *parameter)
{
	unsigned int index;
	int length = 0;

	(void)parameter;
	for (index = 0; index < ARRAY_SIZE(lssbi_extensions); index++)
		length += scnprintf(buffer + length,
				    LSSBI_EXTENSIONS_BUFFER_SIZE - length,
				    "%s %ld %ld\n",
				    lssbi_extensions[index].name,
				    lssbi_extension_results[index].error,
				    lssbi_extension_results[index].value);

	return length;
}

static const struct kernel_param_ops lssbi_extensions_ops = {
	.get = lssbi_param_get_extensions,
};

module_param_cb(extensions, &lssbi_extensions_ops, NULL, 0444);
MODULE_PARM_DESC(extensions, "SBI extension probe results");

static unsigned int lssbi_probe_fwft_features(struct lssbi_sbiret *results)
{
	unsigned int cpu;
	unsigned int index;

	/* All six standard FWFT features are local, so sample one hart. */
	cpu = get_cpu();
	for (index = 0; index < ARRAY_SIZE(lssbi_fwft_features); index++)
		results[index] = lssbi_ecall(LSSBI_EXT_FWFT,
						   LSSBI_EXT_FWFT_GET,
						   lssbi_fwft_features[index].id,
						   0, 0, 0, 0, 0);
	put_cpu();
	return cpu;
}

static int lssbi_param_get_fwft(char *buffer,
				const struct kernel_param *parameter)
{
	struct lssbi_sbiret results[ARRAY_SIZE(lssbi_fwft_features)];
	unsigned int cpu;
	unsigned int index;
	int length = 0;

	(void)parameter;
	cpu = lssbi_probe_fwft_features(results);

	length += scnprintf(buffer + length, LSSBI_FWFT_BUFFER_SIZE - length,
			    "cpu %u\n", cpu);
	for (index = 0; index < ARRAY_SIZE(lssbi_fwft_features); index++)
		length += scnprintf(buffer + length,
				    LSSBI_FWFT_BUFFER_SIZE - length,
				    "%s %ld %ld\n",
				    lssbi_fwft_features[index].name,
				    results[index].error,
				    results[index].value);

	return length;
}

static const struct kernel_param_ops lssbi_fwft_ops = {
	.get = lssbi_param_get_fwft,
};

module_param_cb(fwft, &lssbi_fwft_ops, NULL, 0444);
MODULE_PARM_DESC(fwft, "Live SBI FWFT feature sample");

static int __init lssbi_probe_init(void)
{
	unsigned int index;
	int ret;

	ret = lssbi_read_base(SBI_EXT_BASE_GET_SPEC_VERSION, &spec_version);
	ret |= lssbi_read_base(SBI_EXT_BASE_GET_IMP_ID, &impl_id);
	ret |= lssbi_read_base(SBI_EXT_BASE_GET_IMP_VERSION, &impl_version);
	ret |= lssbi_read_base(SBI_EXT_BASE_GET_MVENDORID, &mvendorid);
	ret |= lssbi_read_base(SBI_EXT_BASE_GET_MARCHID, &marchid);
	ret |= lssbi_read_base(SBI_EXT_BASE_GET_MIMPID, &mimpid);
	if (ret)
		return ret;

	for (index = 0; index < ARRAY_SIZE(lssbi_extensions); index++)
		lssbi_extension_results[index] =
			lssbi_ecall(SBI_EXT_BASE, SBI_EXT_BASE_PROBE_EXT,
				     lssbi_extensions[index].id, 0, 0, 0, 0, 0);

	return 0;
}

static void __exit lssbi_probe_exit(void)
{
}

module_init(lssbi_probe_init);
module_exit(lssbi_probe_exit);

MODULE_LICENSE("GPL");
MODULE_VERSION("0.1.0");
MODULE_DESCRIPTION("RISC-V SBI environment probe for lssbi");
