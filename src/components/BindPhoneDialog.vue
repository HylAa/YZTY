<template>
  <van-dialog
    :show="show"
    @update:show="$emit('update:show', $event)"
    title="绑定手机号"
    show-cancel-button
    :confirm-button-loading="submitting"
    :confirm-button-disabled="submitting"
    @confirm="onConfirm"
  >
    <van-form @submit.prevent="onConfirm">
      <van-cell-group inset>
        <van-field
          v-model="phone"
          label="手机号"
          placeholder="请输入11位手机号"
          type="tel"
          :rules="[
            { required: true, message: '请输入手机号' },
            { validator: validatePhone, message: '手机号格式不正确' }
          ]"
        />
      </van-cell-group>
      <p class="tips">手机号仅用于身份识别与课程绑定，请确认输入正确。</p>
    </van-form>
  </van-dialog>
</template>

<script>
import { ref, watch } from 'vue';
import { showToast } from 'vant';
import api from '../api';

export default {
  name: 'BindPhoneDialog',
  props: {
    show: { type: Boolean, default: false },
    openid: { type: String, default: '' },
  },
  emits: ['update:show', 'bind-success'],
  setup(props, { emit }) {
    const phone = ref('');
    const submitting = ref(false);

    const validatePhone = (val) => /^1\d{10}$/.test(val);

    const onConfirm = async () => {
      if (submitting.value) return;
      const normalizedPhone = phone.value.trim();
      if (!validatePhone(normalizedPhone)) {
        showToast('请输入正确手机号');
        return;
      }
      if (!props.openid) {
        showToast('缺少微信用户标识，请重新授权');
        return;
      }
      submitting.value = true;
      try {
        const res = await api.wechat.bindPhone({
          openid: props.openid,
          phone: normalizedPhone,
        });
        if (res.code !== 0) {
          throw new Error(res.message || '绑定失败');
        }
        emit('bind-success', {
          phoneNumber: normalizedPhone,
          user: res.data,
        });
        emit('update:show', false);
        phone.value = '';
      } catch (error) {
        showToast(error.message || '绑定失败，请稍后再试');
      } finally {
        submitting.value = false;
      }
    };

    watch(() => props.show, (val) => {
      if (!val) {
        phone.value = '';
        submitting.value = false;
      }
    });

    return { phone, validatePhone, onConfirm, submitting };
  }
};
</script>

<style scoped>
.tips {
  margin: 12px 16px 0;
  font-size: 12px;
  color: #969799;
}
</style>
