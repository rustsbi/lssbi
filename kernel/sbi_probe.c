// SPDX-License-Identifier: GPL-2.0-only
#include <linux/errno.h>
#include <linux/init.h>
#include <linux/module.h>
#include <asm/sbi.h>

static long spec_raw;
static long impl_id;
static long impl_version;
static long mvendorid;
static long marchid;
static long mimpid;
#define MAX_EXTENSIONS 32
static int extension_ids[MAX_EXTENSIONS];
static unsigned int extension_id_count;
static long extension_values[MAX_EXTENSIONS];
static unsigned int extension_value_count;
#define MAX_FWFT_FEATURES 16
static int fwft_ids[MAX_FWFT_FEATURES];
static unsigned int fwft_id_count;
static long fwft_errors[MAX_FWFT_FEATURES];
static unsigned int fwft_error_count;
static long fwft_values[MAX_FWFT_FEATURES];
static unsigned int fwft_value_count;

module_param(spec_raw, long, 0444);
MODULE_PARM_DESC(spec_raw, "Raw SBI specification version");
module_param(impl_id, long, 0444);
MODULE_PARM_DESC(impl_id, "SBI implementation ID");
module_param(impl_version, long, 0444);
MODULE_PARM_DESC(impl_version, "Raw SBI implementation version");
module_param(mvendorid, long, 0444);
MODULE_PARM_DESC(mvendorid, "Raw RISC-V machine vendor ID");
module_param(marchid, long, 0444);
MODULE_PARM_DESC(marchid, "Raw RISC-V machine architecture ID");
module_param(mimpid, long, 0444);
MODULE_PARM_DESC(mimpid, "Raw RISC-V machine implementation ID");
module_param_array(extension_ids, int, &extension_id_count, 0400);
MODULE_PARM_DESC(extension_ids, "SBI extension IDs to probe");
module_param_array(extension_values, long, &extension_value_count, 0444);
MODULE_PARM_DESC(extension_values, "SBI extension probe results");
module_param_array(fwft_ids, int, &fwft_id_count, 0400);
MODULE_PARM_DESC(fwft_ids, "SBI FWFT feature IDs to query");
module_param_array(fwft_errors, long, &fwft_error_count, 0444);
MODULE_PARM_DESC(fwft_errors, "SBI FWFT get errors");
module_param_array(fwft_values, long, &fwft_value_count, 0444);
MODULE_PARM_DESC(fwft_values, "SBI FWFT configuration values");

static int __init sbi_probe_init(void)
{
	bool fwft_supported = false;
	struct sbiret ret;
	unsigned int i;

	spec_raw = __sbi_base_ecall(SBI_EXT_BASE_GET_SPEC_VERSION);
	impl_id = __sbi_base_ecall(SBI_EXT_BASE_GET_IMP_ID);
	impl_version = __sbi_base_ecall(SBI_EXT_BASE_GET_IMP_VERSION);
	mvendorid = __sbi_base_ecall(SBI_EXT_BASE_GET_MVENDORID);
	marchid = __sbi_base_ecall(SBI_EXT_BASE_GET_MARCHID);
	mimpid = __sbi_base_ecall(SBI_EXT_BASE_GET_MIMPID);

	if (spec_raw < 0 || impl_id < 0 || impl_version < 0)
		return -EOPNOTSUPP;

	extension_value_count = extension_id_count;
	for (i = 0; i < extension_id_count; i++) {
		extension_values[i] = sbi_probe_extension(extension_ids[i]);
		if (extension_ids[i] == SBI_EXT_FWFT && extension_values[i] > 0)
			fwft_supported = true;
	}

	if (fwft_supported) {
		fwft_error_count = fwft_value_count = fwft_id_count;
		for (i = 0; i < fwft_id_count; i++) {
			ret = sbi_ecall(SBI_EXT_FWFT, SBI_EXT_FWFT_GET,
					fwft_ids[i], 0, 0, 0, 0, 0);
			fwft_errors[i] = ret.error;
			fwft_values[i] = ret.value;
		}
	}

	return 0;
}

static void __exit sbi_probe_exit(void)
{
}

module_init(sbi_probe_init);
module_exit(sbi_probe_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("One-shot RISC-V SBI information probe");
