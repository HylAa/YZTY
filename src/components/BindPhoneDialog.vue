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
          :rules="[{ required: true, message: '请输入手机号' }, { validator: validatePhone, message: '手机号格式不正确' }]"
        />
        <van-field
          v-model="code"
          label="验证码"
          placeholder="请输入验证码"
          type="digit"
          maxlength="6"
        >
          <template #button>
            <van-button size="small" type="primary" :disabled="countdown>0" @click="sendCode">
              {{ countdown>0 ? `${countdown}s` : '发送验证码' }}
            </van-button>
          </template>
        </van-field>
      </van-cell-group>
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
    const code = ref('');
    const countdown = ref(0);
    const submitting = ref(false);
    let timer = null;

    const validatePhone = (val) => /^1\d{10}$/.test(val);

    const sendCode = async () => {
      if (!validatePhone(phone.value)) {
        showToast('请输入正确手机号');
        return;
      }
      // TODO: 调用后端发送短信验证码接口 /api/sms/send
      showToast('验证码已发送');
      countdown.value = 60;
      timer = setInterval(() => {
        countdown.value--;
        if (countdown.value <= 0) {
          clearInterval(timer);
          timer = null;
        }
      }, 1000);
    };

    const onConfirm = async () => {
      if (submitting.value) return;
      const normalizedPhone = phone.value.trim();
      if (!validatePhone(normalizedPhone)) {
        showToast('请输入正确手机号');
        return;
      }
      if (!code.value || code.value.length < 4) {
        showToast('请输入有效验证码');
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
        code.value = '';
        if (timer) clearInterval(timer);
        countdown.value = 0;
      } catch (error) {
        showToast(error.message || '绑定失败，请稍后再试');
      } finally {
        submitting.value = false;
      }
    };

    watch(() => props.show, (val) => {
      if (!val) {
        phone.value = '';
        code.value = '';
        if (timer) clearInterval(timer);
        countdown.value = 0;
        submitting.value = false;
      }
    });

    return { phone, code, countdown, validatePhone, sendCode, onConfirm, submitting };
  }
};
</script>
