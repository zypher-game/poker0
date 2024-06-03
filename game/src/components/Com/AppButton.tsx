import { message, notification } from 'antd';
import React, { HTMLAttributes, HTMLProps, useCallback, useContext, useEffect, useState } from 'react';
import styled from 'styled-components';
import { LoadingOutlined } from '@ant-design/icons';
import classNames from 'classnames';
import { errorParse } from 'src/utils';
import { GlobalVar } from 'src/constants';

interface CptTypes {
  children?: React.ReactNode;
  className?: string;
  onClick?: (arg?: any) => any;
  loading?: boolean;
  link?: string;
  style?: React.CSSProperties;
  disabled?: boolean | null;
  disabledClick?: boolean;
  id?: string;
  size?: 'mini';
  type?: 'primary';
}

export const AppButton: React.FC<CptTypes> = React.memo(function AppButton(props) {
  props = { ...props };
  const [loadingTemp, _loading] = useState(false);
  const loading = props.loading || loadingTemp;

  const onClick = useCallback(async () => {
    if (loading) return;
    if (props.disabled && props.disabledClick !== true) return;
    if (!props.onClick) return;
    _loading(true);
    try {
      await props.onClick();
    } catch (e) {
      console.log('appButton catch', e);
      const tip = errorParse(e);
      if (!tip) return;
      GlobalVar.notification.error({ message: 'Error', description: tip });
    } finally {
      _loading(false);
    }
  }, [props.onClick, loading]);

  const linkProps = props.link ? { href: props.link, target: '_blank', rel: 'noreferrer' } : null;

  return (
    <AppButtonStyle
      onClick={onClick}
      {...linkProps}
      style={props.style}
      id={props.id}
      className={classNames(`type-${props.type || 'primary'}`, `size-${props.size}`, { loading, disabled: props.disabled }, props.className)}
    >
      {props.children}
      {loading && <LoadingOutlined />}
    </AppButtonStyle>
  );
});

const ButtonStyle = (props: React.AnchorHTMLAttributes<HTMLAnchorElement> | React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>) => {
  if ('href' in props) return <a {...props} />;
  return <div {...(props as any)} />;
};

const AppButtonStyle = styled(ButtonStyle)`
  cursor: pointer;
  position: relative;
  transition: all ease 0.2s;
  user-select: none;
  transition: all 0.2s linear;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  &.type-primary {
    color: #fefefe96;
    border: 4px solid #ffffff3b;
    border-radius: 8px;
    background-color: #882db4;
    &:hover {
      border: 4px solid #ffffff66;
      border-radius: 16px;
    }
  }
  &.disabled {
    filter: grayscale(100%);
    cursor: not-allowed;
    pointer-events: none;
  }
  &:hover {
    opacity: 0.9;
  }
  &.loading {
    cursor: wait;
  }
`;
